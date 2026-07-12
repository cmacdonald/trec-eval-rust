# `te-rust` Design and Implementation Issues

This document tracks active design questions, structural decisions, and implementation tasks for the `te-rust` project. Issues are categorized by type and updated as we progress.

---

## Active Issues

### 15. Lack of Design for Preference-Based Evaluations
*   **Type**: Design (Design Stage)
*   **Status**: Active
*   **Description**: C's preference-based evaluation (`form_prefs_counts.c`, ~1,200 lines) implements transitive closures of partial preferences, equivalence class partitioning, and complex zone counting across five topological areas (A1–A5). `EVAL_DESIGN.md` completely glosses over this, declaring only an undefined `Prefs(PrefsEvalState)`. We need a robust mathematical design for porting this logic to Rust.

### 16. The `bogus_ranking` Ingestion Hack
*   **Type**: Design (Design Stage)
*   **Status**: Active
*   **Description**: For missing queries under `-c`, C `trec_eval` creates a `bogus_ranking` containing exactly one non-relevant document (`ceci_nest_pas_un_docno`) to avoid division by zero. C then runs a hardcoded post-evaluation override loop to reset corrupted counts (like `num_ret` and `utility`). `te-rust` needs to define how it handles missing queries natively without these hacks while preserving exact equivalence.

### 17. Defensive `rel_levels` Bounds Sizing
*   **Type**: Design (Design Stage)
*   **Status**: Active
*   **Description**: In C, `rel_levels` is dynamically allocated based on the maximum relevance score in the qrels (`max_rel + 1`). If `max_rel < relevance_level`, metrics like `bpref` still loop up to `relevance_level`, causing an out-of-bounds read. In Rust, this will cause thread panics. `te-rust` must defensively size `rel_levels` or use safe default-value lookups.

### 19. Fragile Testing Harness: `cargo run` and Exact String Comparisons
*   **Type**: Testing (Design Stage)
*   **Status**: Active
*   **Description**: `TEST_DESIGN.md` invokes `cargo run --bin te-rust` recursively, which blocks cargo locks and is extremely slow during parallel execution. It should use `env!("CARGO_BIN_EXE_te-rust")`. Furthermore, strict line-by-line string comparison will fail on minor floating-point rounding variations or spacing differences. We need a relational regression parser that compares structured triples with a numerical epsilon.

---

## Resolved Issues

### 1. Unified vs. Hierarchical Flat Arrays (De-duplication of Query IDs)
*   **Type**: Design
*   **Status**: Resolved (Option B)
*   **Description**: Resolved to use hierarchical structs (`RunQuery`, `QrelsQuery`) to group records by `qid`, avoiding duplicate query ID string allocations and naturally aligning with C's query grouping.

### 2. Stream-based Reading and I/O Genericity
*   **Type**: Design
*   **Status**: Resolved (Option B)
*   **Description**: Decided to use standard `std::io::BufRead` trait for parser inputs. This decouples the core parsing logic from physical file opening, allows simple unit testing using in-memory byte streams, and directly enables parsing from Standard Input (`stdin`).

### 3. Immediate Abort vs. Collected Errors during Parsing
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Decided to abort parsing immediately on the first syntax error. To compensate for not collecting errors, we will heavily prioritize highly detailed and helpful error diagnostics (including line numbers, raw line contents, expected format, and the reason for parsing failure).

### 4. Sorting non-Ord floats (`f64`) in Rust
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to explicitly validate during parsing that all similarity scores are valid finite numbers (rejecting `NaN` and `Infinity` with an error). This guarantees that `.partial_cmp().unwrap()` is entirely safe and infallible for sorting without needing third-party wrapper dependencies.

### 5. Configuration Metadata in Comments
*   **Type**: Design/Future Feature
*   **Status**: Resolved (Option A - TOML Blocks)
*   **Description**: File comments starting with `#` can contain inline configuration metadata formatted as a TOML block. The ingestion phase strips the leading `#` (and up to one space) from comment lines and saves them to a flat vector of comments. A post-parser metadata resolver scans these cleaned comments for a `[te-rust.config]` block, deserializes it using standard TOML libraries, and merges it into the execution parameters with a strict hierarchy of precedence: Default Settings < File-Specified Config < User CLI Arguments.


### 6. Double-Precision Floating-Point Compatibility (Rust vs C)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to use standard IEEE 754 64-bit float math (`f64`) natively, which maps to identical CPU-level instructions in modern compiled targets. If tiny floating-point rounding variations or transcendental libm math changes emerge during differential integration testing, we will first verify the Rust metric math for 100% correct implementation. Upon confirmation of correctness, we will either update our local quicktest expected output files to match Rust's highly precise calculations, or we will enhance our regression test runner to parse the tables and check scores with a numeric epsilon (e.g., `1e-5`) rather than checking strict string equality.

### 7. Evaluation Corner Cases (Empty Runs, Zero-Relevance Topics)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to define mathematically safe default values individually for each metric under boundary cases (such as empty rankings, zero relevant documents, etc.), precisely aligning with the legacy defaults established by `trec_eval` (typically `0.0`, but varies where alternative baseline formulations exist). All calculators will defensively check denominators to prevent any generation of `NaN` or `Infinity` values, and our centralized boundary test suite will automatically run these edge cases against every metric to verify absolute safety and correctness.


### 8. Printing and Formatting Types for Measures
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to define a standard `ValueFormat` enum containing `Float`, `Integer`, `Str`, and `QuotedStr` variants. Each implementation of the `Measure` trait is required to expose its desired format via a trait method (e.g., `fn format(&self) -> ValueFormat`). The printing presentation layer will automatically format outputs accordingly, completely decoupling calculation code from stdout formatting.

### 9. Representation and Printing of String-Valued Measures (e.g., `runid`, `relstring`)
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to introduce a standard `MetricValue` enum containing `Float(f64)`, `Integer(i64)`, and `Str(String)` variants. The `calc` method of our `Measure` trait returns this structured enum. This allows string-valued metrics (like `runid` and `relstring`) to be calculated thread-safely and returned directly to the harness without any dangerous C-style pointer-casting or global variable hacks. The print formatter formats them using `ValueFormat`, including support for wrapping string sequences in single quotes (as required for backwards-compatible `relstring` outputs).


### 10. Metric Compatibility with Ground-Truth Formats via Traits
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to use a single unified `Measure` trait exposing an `eval_type(&self) -> EvaluationType` method (returning `Standard`, `Preferences`, or `JudgmentGroups`). To optimize execution, the validation pass is performed **immediately after the ground-truth (qrels) file is loaded and parsed**, since the ground-truth format dictates the entire evaluation paradigm. If any requested measure is incompatible with the parsed ground-truth format, the harness aborts instantly with a clean diagnostic error before wasting I/O or memory opening and parsing the potentially multi-gigabyte run results file.


### 11. Regression Testing Harness in Rust
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to implement the regression check suite directly inside Rust's standard integration testing framework as `tests/regression.rs`. The test runner copies/references the regression files inside the Cargo package, compiles the `te-rust` binary, and executes it as a child process using `std::process::Command` across the 13 quicktest scenarios. It captures the stdout and compares it with the expected C outputs (out.*) line-by-line, providing highly detailed diff context on mismatch while integrating flawlessly with `cargo test` for local development and CI pipelines.


### 12. Avoiding Boundary Test Duplication via Shared Declarative Macros
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to create a standard `test_measure_boundaries!` macro in our test common module, coupled with a helper generator `make_mock_state`. This enables metrics to cleanly declare their own specific boundary condition matrices in a highly compact, data-driven way, completely eliminating duplicate mock state configuration and assertion loops.


### 13. Enforcing Uniform Boundary Testing across all Metrics
*   **Type**: Design
*   **Status**: Resolved (Option A)
*   **Description**: Resolved to implement an automated boundary checking suite inside `tests/uniform_boundaries.rs`. This suite automatically queries our central measure registry (`get_all_measures()`) and runs all registered standard measures against core boundary scenarios (e.g. empty ranking, zero-relevance topic). It asserts universal mathematical invariants (no non-finite values, standard float metrics defaulting safely to zero, integer counts defaulting to 0) dynamically, guaranteeing complete test coverage for all current and future metrics automatically.

### 14. Stateless `Measure` Trait vs. Parameter Mutation
*   **Type**: Design
*   **Status**: Resolved (Stateless Factory Pattern)
*   **Description**: Resolved to refactor the `Measure` trait to be completely stateless and thread-safe. Instead of calling a mutable `init(&self)` method on a single pre-allocated metric instance, individual metric instances are instantiated up front with their specific parsed parameters (e.g. cutoffs) as immutable fields. The trait's `sub_metrics(&self)` and `initial_values(&self)` methods expose these pre-computed configurations to the evaluation harness, enabling efficient, lock-free parallel execution.

### 20. Infallible Float Sorting and Determinism
*   **Type**: Design
*   **Status**: Resolved (total_cmp & Measure-Dependent guards)
*   **Description**: Resolved to incorporate `f64::total_cmp` for high-performance, branchless, and crash-safe document sorting within the alignment engine, coupled with strict fail-fast validation in the parser to reject `NaN` and `Infinity` float inputs. Furthermore, recognized that "correct fallback score" under undefined states is measure-dependent (e.g., 0.0 vs utility offsets); resolved that each individual metric `Measure` implementation must defensively evaluate its own specific mathematical boundaries and return its designated standard fallback rather than propagating raw floating-point `NaN` or `Infinity` values.

### 18. Asymmetric Comment Handling
*   **Type**: Design
*   **Status**: Resolved (Option B - Relaxed Quality-of-Life)
*   **Description**: Resolved to adopt Option B (relaxed quality-of-life parsing), which trims leading whitespace before checking for `#` comment characters. Since comment support is a highly recent addition in `trec_eval`, backward-compatibility of files from `te-rust` to old legacy C versions is not a primary concern. Allowing leading spaces before comments provides a much better and more forgiving user experience for manual file editing, avoiding fatal parsing failures on minor indentation variations.
