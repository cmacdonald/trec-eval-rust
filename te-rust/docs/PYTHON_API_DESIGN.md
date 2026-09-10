# Python API & Bindings Design Document for te-rust

## 1. Overview and Motivation

This document presents the architectural design for exposing `te-rust` functionality as a native Python package. The Python ecosystem is the primary working environment for modern Information Retrieval (IR) research, machine learning rankers, and data science workflows.

### Goals
1. **High Performance & Safety**: Leverage Rust's speed, zero-cost abstractions, and thread safety with zero memory leaks and safe GIL release during batch evaluation.
2. **Compatibility & Migration Ease**: Provide drop-in or near-drop-in compatibility with legacy bindings (`pytrec_eval`), while offering seamless interop for modern ecosystems (`ir_measures`, `trectools`).
3. **Versatile Input/Output Formats**: Ingest data from files, in-memory nested dictionaries, row iterables, custom user Python objects, and tabular structures (Pandas/Polars/Arrow).
4. **Zero Heavy Mandatory Dependencies**: Support DataFrames and scientific arrays without making `pandas`, `pyarrow`, or `numpy` hard installation dependencies.
5. **Extensibility & User Object Bridging**: Allow Python users to evaluate custom Python objects and classes (search hits, dataclasses, pydantic models) without manual pre-conversion to intermediate dictionaries.

---

## 2. Analysis of Existing Python Integrations

| Feature / Library | `pytrec_eval` | `ir_measures` | `trectools` | `sigtrec_eval` |
| :--- | :--- | :--- | :--- | :--- |
| **Backend** | C `trec_eval` via Cython/C API | Multi-provider (Cython, Python, `pytrec_eval`, etc.) | Pure Python / Pandas | Wrapper around `trec_eval` CLI |
| **Primary Input Format** | `Dict[str, Dict[str, Union[int, float]]]` | Dicts, DataFrames, Iterators of namedtuples/dataclasses | File paths or Pandas DataFrames | File paths |
| **Primary Output Format** | Nested `Dict[str, Dict[str, float]]` | Namedtuple / Dataclass stream (`Metric(query_id, measure, value)`) | Pandas DataFrames (`TrecRes`) | CLI text / significance summary dicts |
| **Dependencies** | Minimal / None (C extension) | `cffi`, optional `pandas`, `scipy` | Heavy (`pandas`, `scipy`, `numpy`, `matplotlib`) | `scipy`, `numpy`, `trec_eval` binary |
| **Strengths** | Simple, ubiquitous, fast for small runs | Composable measure objects (`nDCG@10`), provider-agnostic | Rich IR campaign tools (fusion, pooling) | Statistical significance testing |
| **Weaknesses** | Dict conversion is high overhead; no DataFrame or zero-copy support; hard to maintain C code | Measure algebra adds complexity for simple CLI parity | High memory and slow parsing for large runs | Subprocess CLI overhead |

---

## 3. Rust-Python Binding Architecture

### 3.1 Tooling & Crate Organization

We propose using **PyO3** + **Maturin**:
- **PyO3**: Industry standard for Rust-Python bindings; generates efficient CPython C-extension modules with automatic type conversion, exception mapping, and GIL management.
- **Maturin**: Build tool and PEP 517 build backend for compiling and packaging PyO3 crates into Python wheels for PyPI.

#### Proposed Workspace Structure
Instead of coupling Python bindings directly into the standalone CLI binary crate, we organize `te-rust` as a Cargo workspace:

```text
trec-eval-rust/
├── te-rust/                  # Core Rust crate (lib + CLI binary)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Core evaluation engine, types, parsers, metrics
│       ├── main.rs           # CLI executable
│       ├── io/
│       ├── eval/
│       └── metrics/
├── te-python/                # Python binding crate (PyO3)
│   ├── Cargo.toml            # depends on path = "../te-rust"
│   ├── pyproject.toml        # maturin configuration, package metadata
│   ├── src/
│   │   ├── lib.rs            # PyO3 module definition & PyO3 classes
│   │   ├── conversions.rs    # Input adaptation (dicts, dataframes, iterators)
│   │   ├── evaluator.rs      # Evaluator PyClass
│   │   └── compat.rs         # Compatibility layers (pytrec_eval, ir_measures)
│   └── python/
│       └── trec_eval/        # Pure Python package layer / stubs / typing
│           ├── __init__.py
│           ├── compat/
│           │   ├── pytrec_eval.py
│           │   └── ir_measures.py
│           └── py.typed
```

**Pros of Separation**:
- The core CLI binary remains lean with zero Python C headers or runtime linkage.
- The Python bindings can be versioned, tested, and released independently or alongside the CLI.
- Core Rust engine can be consumed by other Rust projects or WASM targets.

---

## 4. Input Data Ingestion & Bridging Python Objects

A critical bottleneck in existing tools like `pytrec_eval` is the FFI boundary: converting Python dicts of dicts into C/Rust data structures requires traversing millions of Python heap objects, creating substantial GC and serialization overhead.

We design a multi-path ingestion engine in Rust:

```mermaid
flowchart TD
    A[Input Data Source] --> B{Type Detection}
    B -->|File Path / str / PathLike| C[Fast Rust Native Parser]
    B -->|Nested Dicts| D[Dict Ingestor: Dict[str, Dict[str, num]]]
    B -->|Pandas / Polars / Arrow| E[Columnar / Arrow Ingestor]
    B -->|Iterable of Objects / Tuples| F[Duck-Typing / Protocol Ingestor]
    
    C --> G[Internal Rust Qrels & Run Storage]
    D --> G
    E --> G
    F --> G
    
    G --> H[Parallel Metric Computation Engine Rayon]
    H --> I{Output Mode}
    I -->|Dict| J[Nested Dict: query -> measure -> score]
    I -->|Records| K[List of Metric objects]
    I -->|DataFrame| L[Pandas / Polars DataFrame]
```

### 4.1 Ingestion Modes

1. **File Paths (`str` or `os.PathLike`)**:
   - Direct Rust file I/O using buffered readers.
   - Bypasses Python interpreter completely; ideal for large TREC runs.

2. **Nested Dictionaries (Legacy `pytrec_eval` compatibility)**:
   - `Dict[str, Dict[str, Union[int, float]]]`
   - Direct iteration over Python dictionaries via PyO3 without full JSON intermediate serialization.

3. **Row Sequences / Iterables of Tuples**:
   - `Iterable[Tuple[str, str, float]]` (e.g. `(query_id, doc_id, score)`)
   - Low-overhead ingestion for custom Python database cursors or generators.

4. **Arbitrary Python Objects / Classes (Duck Typing / Protocol)**:
   - Supports custom classes (e.g. `SearchHit`, dataclasses, Pydantic models).
   - Ingestor checks for expected attribute names via fast PyO3 attribute lookups:
     - Query ID: `.query_id`, `.qid`, `.topic_id`, or `[0]`
     - Document ID: `.doc_id`, `.docno`, `.document_id`, or `[1]` / `[2]`
     - Score / Rank / Rel: `.score`, `.rank`, `.relevance`, `.rel`
   - Alternatively, allow user-supplied extractor functions/lambdas:
     `evaluator.evaluate(run_docs, qid_fn=lambda x: x.qid, doc_fn=lambda x: x.doc_id, score_fn=lambda x: x.score)`

5. **DataFrames & Columnar Buffers (Zero / Low-Copy)**:
   - Supports Pandas DataFrames, Polars DataFrames, and PyArrow Tables.
   - Rust extracts pointers or contiguous column arrays (`query_id`, `doc_id`, `score`/`rel`) without converting individual rows into Python objects.

---

## 5. Zero-Mandatory-Dependency DataFrame & Scientific Stack Strategy

### 5.1 The Problem & Opportunity
IR researchers reflexively use **NumPy, Pandas, and SciPy** for model inference (e.g. dense retrieval scores in float arrays), experiment analysis, and statistical significance testing.
However:
- Making `pandas`, `scipy`, and `numpy` hard dependencies in `pyproject.toml` forces a heavy install (~150MB+) on users who only want lightweight evaluation.
- Conversely, requiring researchers to manually convert NumPy matrices or DataFrames into nested Python dicts creates massive CPU and memory overhead.

### 5.2 NumPy & Columnar Ingestion (Zero-Copy FFI)
Through Python's Buffer Protocol (PEP 3118) and PyO3's buffer interfaces:
1. **Direct Array Ingestion**:
   - `evaluator.evaluate_arrays(query_ids, doc_ids, scores)` where `scores` is a 1D contiguous NumPy array (or PyTorch / CuPy array converted to CPU buffer).
   - Rust can read the raw C-contiguous memory slice of `f32` or `f64` floats directly without allocating intermediate Python objects.
2. **Dense Score Matrices for Neural Rankers**:
   - `evaluator.evaluate_matrix(query_ids, doc_ids, score_matrix)` where `score_matrix` is `(num_queries, num_docs)`.

### 5.3 Pandas Integration: Wide vs Tidy Formats
1. **Input Detection**:
   - Accepts any DataFrame with columns `query_id` (or `qid`), `doc_id` (or `docno`), and `score` (for runs) or `relevance` (for qrels).
   - Ingests columns directly as contiguous series buffers rather than row-by-row iteration.
2. **Output Formatting**:
   - `.to_dataframe(format="wide")`: Query IDs as rows, measures as columns (convenient for machine learning features, correlational analysis).
   - `.to_dataframe(format="tidy")`: Long format `(query_id, measure, value)` (convenient for Seaborn/Altair plotting and `groupby`).
   - `.to_numpy(measure="map")`: Returns a 1D NumPy float array of per-query scores with guaranteed deterministic topic ordering.

### 5.4 Multi-Run Statistical Significance & Multiple Comparisons Corrections

In IR experimental campaigns (TREC tracks, SIGIR/CIKM benchmarking, model ablation studies), researchers frequently evaluate tens or hundreds of runs across multiple evaluation measures. Evaluating multiple hypotheses without adjusting for multiple comparisons leads to severe Type I error inflation (false discoveries).

#### 5.4.1 Division of Responsibilities (Rust vs. Python)
We split the significance testing pipeline to maximize speed and analytical flexibility:

1. **Rust Core (High-Speed Raw Pairwise Statistics)**:
   - Computes aligned topic score vectors with deterministic ordering and zero-fill for missing topics.
   - Calculates raw test statistics and unadjusted p-values:
     - **Paired Student's t-test**: Fast parametric test for mean differences.
     - **Wilcoxon Signed-Rank Test**: Non-parametric test for median ranking differences.
     - **Randomized Permutation / Fisher Randomization Test**: Exact or Monte Carlo ($B=10{,}000$ to $100{,}000$ resamples) permutation test.
     - **Studentized Bootstrap**: Confidence intervals and bootstrap p-values.
   - **Parallel Batch Execution**: When evaluating $N$ runs ($\binom{N}{2}$ pairwise comparisons) across $M$ measures, Rust evaluates all $\binom{N}{2} \times M$ tests concurrently across CPU cores via Rayon in milliseconds.

2. **Python Layer (Multiple Comparisons Adjustments & Reporting)**:
   - Ingests the matrix of raw p-values from Rust and applies multiple testing corrections:
     - **FWER Control (Family-Wise Error Rate)**:
       - `holm` (Holm-Bonferroni step-down) — *Recommended default: controls FWER with substantially higher power than standard Bonferroni*.
       - `bonferroni` (Single-step Bonferroni $\alpha / m$).
       - `hochberg` (Hochberg step-up procedure).
       - `sidak` / `holm_sidak` (Exact Šidák adjustment under independence).
     - **FDR Control (False Discovery Rate)**:
       - `fdr_bh` (Benjamini-Hochberg) — *Standard for large-scale multi-run IR evaluations*.
       - `fdr_by` (Benjamini-Yekutieli) — *FDR control under arbitrary dependence*.
     - `none` — Raw unadjusted p-values.

#### 5.4.2 Python Multi-Run Comparison API

1. **One-vs-Many (Treatment vs Baselines)**:
   ```python
   evaluator = trec_eval.Evaluator(qrels, measures=["map", "ndcg@10"])
   
   # Compare our new model against 5 baseline runs with Benjamini-Hochberg FDR correction
   comp_table = evaluator.compare_against_baseline(
       baseline="runs/bm25.txt",
       candidates={
           "splade": "runs/splade.txt",
           "colbert": "runs/colbert.txt",
           "cross_encoder": "runs/rerank.txt"
       },
       test="permutation",          # executed in high-speed Rust
       correction="holm",           # adjusted in Python layer
       alpha=0.05
   )
   
   print(comp_table.to_dataframe())
   # Output columns: [system, measure, baseline_score, system_score, diff, p_raw, p_adj, significant]
   ```

2. **All-Pairs Comparison Matrix**:
   ```python
   # Full pairwise comparison matrix across all runs
   matrix = evaluator.compare_all(
       runs={"bm25": run1, "splade": run2, "colbert": run3},
       measure="map",
       test="paired_t",
       correction="fdr_bh"
   )
   print(matrix.pvalues_adjusted)   # Symmetric DataFrame of adjusted p-values
   print(matrix.summary_table())     # Printable summary with significance markers (▲ / ▼ / ns)
   ```

3. **Single Pair Comparison**:
   ```python
   comp = res_a.compare(res_b, measure="map", test="paired_t")
   print(comp.pvalue, comp.statistic, comp.significant(0.05))
   ```

### 5.5 Packaging Extras
In `pyproject.toml`:
```toml
[project.optional-dependencies]
numpy = ["numpy>=1.20"]
pandas = ["pandas>=1.0", "numpy>=1.20"]
stats = ["scipy>=1.8", "numpy>=1.20"]
all = ["pandas", "numpy", "scipy", "polars", "pyarrow"]
```

---

## 6. API Design & Migration Patterns

We propose a three-tiered API:

### 6.1 Tier 1: Modern Object-Oriented API (`trec_eval.Evaluator`)

Designed for high efficiency when evaluating multiple runs against the same qrels (qrels parsed/indexed once in Rust):

```python
from trec_eval import Evaluator

# Initialize evaluator with qrels (from file, dict, or DataFrame)
evaluator = Evaluator(
    qrels="path/to/qrels.txt",  # or qrels_dict, or qrels_df
    measures=["map", "ndcg_cut.10", "P.5,10", "recip_rank"],
    relevance_level=1,
    judged_docs_only=False
)

# Evaluate a run (from file, dict, DataFrame, or custom object list)
results = evaluator.evaluate(run)

# Per-query and aggregate access
print(results.aggregate())           # {"map": 0.42, "ndcg_cut_10": 0.58, ...}
print(results.per_query())          # {"q1": {"map": 0.5, ...}, "q2": {...}}
df = results.to_dataframe()         # Returns pandas DataFrame if pandas is installed
```

### 6.2 Tier 2: Convenient Functional API (`trec_eval.evaluate`)

One-liner for ad-hoc evaluations:

```python
import trec_eval

res = trec_eval.evaluate(
    qrels="qrels.txt",
    run="run.txt",
    measures=["map", "ndcg@10", "bpref"],
    as_dataframe=False  # True to return DataFrame
)
```

### 6.3 Tier 3: Drop-in Compatibility Layer

#### `pytrec_eval` Compatibility
Users can migrate existing code by changing only the import:

```python
# Existing code:
# import pytrec_eval
# New drop-in replacement:
import trec_eval.compat.pytrec_eval as pytrec_eval

evaluator = pytrec_eval.RelevanceEvaluator(qrels_dict, {"map", "ndcg"})
res = evaluator.evaluate(run_dict)  # Exact same dict-of-dict structure and measure names
```

#### `ir_measures` Provider Integration
`ir_measures` allows custom measure providers. We can expose an adapter that registers `te-rust` as an ultra-fast provider for `ir_measures`:

```python
import ir_measures
from trec_eval.compat.ir_measures import TeRustProvider

ir_measures.register_provider(TeRustProvider())
```

---

## 7. Performance & Concurrency Model

1. **GIL Release**:
   - All computationally intensive phases (file parsing, query alignment, metric calculations across topics) release the Python GIL (`py.allow_threads(...)`).
   - Enables multi-threaded Python applications (e.g. concurrent evaluation of multiple runs across worker threads) without GIL contention.
2. **Parallel Evaluation via Rayon**:
   - Metric computation across hundreds or thousands of topics is parallelized across CPU cores using Rayon in Rust.
3. **Qrels Pre-indexing**:
   - In `Evaluator(qrels, ...)`, judgments are parsed, sorted, and converted into Rust lookup structures once.
   - Subsequent `evaluator.evaluate(run)` calls incur zero qrels re-parsing or re-indexing cost.

---

## 8. Measure Specification & Syntax

To ensure compatibility with existing habits while providing modern ergonomics:

1. **Standard `trec_eval` CLI names**:
   `"map"`, `"ndcg"`, `"ndcg_cut.10"`, `"P.5,10,20"`, `"recip_rank"`, `"bpref"`, `"runid"`, etc.
2. **Convenience Aliases**:
   - Cutoff syntax with `@`: `"ndcg@10"` -> `"ndcg_cut.10"`, `"P@5"` -> `"P.5"`.
   - Case-insensitive parsing: `"NDCG@10"` -> `"ndcg_cut.10"`.
3. **Help & Introspection**:
   `trec_eval.list_measures()` and `trec_eval.describe_measure("map")` available directly in Python.

---

## 9. Design Trade-Offs & Questions for Discussion

### 9.1 Evaluation Output Structure
- **Option A (pytrec_eval style)**: Return raw nested dict `{qid: {meas: val}}` and separate summary.
- **Option B (Result Object wrapper)**: Return a rich `EvalResult` object that implements `dict` indexing (for backwards compatibility) but also provides `.aggregate()`, `.per_query()`, `.to_dataframe()`, and `.to_dict()`.
- *Recommendation*: Option B gives full backward compatibility via mapping protocols while enabling clean DataFrame and method access.

### 9.2 Custom Python Object Extraction
- **Option A**: Inspect object attributes via fixed standard names (`.query_id`, `.doc_id`, `.score`).
- **Option B**: Support custom accessor callbacks / lambdas passed to `evaluate(run, qid_getter=..., ...)`.
- **Option C**: Support both (try standard attributes first, allow custom getters if specified).
- *Recommendation*: Option C provides maximum convenience out-of-the-box and full flexibility for idiosyncratic classes.

### 9.3 Package Naming on PyPI
- Existing names in the ecosystem: `pytrec_eval`, `pytrec_eval_terrier`, `trectools`, `ir_measures`.
- Potential candidate package names: `trec-eval`, `te-rust`, `pytrec-eval-rust`.
