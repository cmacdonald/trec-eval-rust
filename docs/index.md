# trec_eval Documentation

`trec_eval` is a Rust reimplementation of the standard TREC evaluation tool(`trec_eval 9.x/10.x`). It provides a command-line interface with exact numerical equivalence to the reference C implementation, as well as a native Python library (`trec-eval`). 

The Python library includes support for running statistical tests and multiple-comparisons corrections on multiple evaluation outputs, facilities for implementing measures in Python, ingest and output to Numpy arrays and Pandas Dataframes, and compatibility with ir_measures and pytrec_eval.

---

## Installation & Optional Dependencies

### Python Package (`trec-eval`)

The Python package has **zero mandatory runtime dependencies**. All core evaluation features, statistical tests (`paired_t`, `permutation`, `bootstrap`), multiple comparisons corrections (`holm`, `fdr_bh`, `bonferroni`), custom measure hooks, and `pytrec_eval` compatibility work out of the box with standard Python.

```bash
# Minimal installation (core evaluation, nested dicts, file inputs, significance tests)
pip install trec-eval

# Optional extras for tabular and scientific array outputs
pip install "trec-eval[pandas]"   # Required for .to_dataframe()
pip install "trec-eval[numpy]"    # Returns numpy.ndarray from .to_numpy()
pip install "trec-eval[all]"      # Installs pandas, polars, and numpy

```

### Feature Support by Dependency

| Feature | Without Extra | With Optional Extra |
| :--- | :--- | :--- |
| **Basic Evaluation** (`evaluate`, `Evaluator`) | Works (dicts, files, row iterables) | Works |
| **Statistical Tests** (`.compare`, `compare_against_baseline`) | Works (pure-Rust backend) | Works |
| **Multiple Testing Corrections** (`adjust_pvalues`) | Works (`holm`, `fdr_bh`, `bonferroni`) | Works |
| **Custom Python Measures** (`Measure`, `@register_measure`) | Works | Works |
| **NumPy Export** (`results.to_numpy()`) | Returns `list[float]` fallback | Returns `numpy.ndarray` (`[numpy]`) |
| **DataFrame Export** (`results.to_dataframe()`) | Raises `ImportError` | Returns `pandas.DataFrame` or `polars.DataFrame` (`[pandas]`) |
| **Multi-Comparison DataFrame** (`comp_table.to_dataframe()`) | Raises `ImportError` | Returns `pandas.DataFrame` (`[pandas]`) |

### Command-Line Executable (`te-rust`)


```bash
# Build from source using Cargo
cargo build --release
```

The compiled binary is placed at `target/release/te-rust`.

---

## Quickstart

=== "Python API"

    ```python
    import trec_eval

    # Relevance judgments: query_id -> {doc_id: relevance}
    qrels = {
        "q1": {"d1": 0, "d2": 1, "d3": 0},
        "q2": {"d2": 1, "d3": 1},
    }

    # System run: query_id -> {doc_id: score}
    run = {
        "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
        "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
    }

    # Evaluate using standard measures
    results = trec_eval.evaluate(qrels, run, measures=["map", "ndcg@10", "recip_rank"])

    # Summary averages across queries
    print(results.aggregate())
    # {'map': 0.4583, 'ndcg_cut_10': 0.5967, 'recip_rank': 0.5833}

    # Per-query breakdown (pytrec_eval mapping format)
    print(results["q1"]["map"])  # 0.3333
    ```

=== "Command-Line Interface"

    ```bash
    # Standard evaluation of official measure set
    te-rust qrels.txt results.txt

    # Print query-level breakdown and specific cutoffs
    te-rust -q -m map -m P.5,10,20 -m ndcg@10 -m recip_rank qrels.txt results.txt

    # 95% bootstrap confidence intervals for summary averages
    te-rust -C --ci-pretty -m official qrels.txt results.txt
    ```

---

## Library Architecture

The project consists of two core components:

```
trec-eval-rust/
├── te-rust/     # Core evaluation engine (parsing, alignment, measures, CLI)
└── te-python/   # PyO3 bindings and Python packaging
```

- **`te-rust` Core Library**: Contains the format parsers, query alignment logic, statistical significance tests, and all 33 evaluation measure implementations. It is built as a stateless evaluation engine.
- **`trec-eval` Python Package**: Provides the `Evaluator` class, `EvalResult` mapping object, custom measure base classes, and statistical comparison methods.
