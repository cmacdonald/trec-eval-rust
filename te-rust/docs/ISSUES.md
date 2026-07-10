# `te-rust` Design and Implementation Issues

This document tracks active design questions, structural decisions, and implementation tasks for the `te-rust` project. Issues are categorized by type and updated as we progress.

---

## Active Issues

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
*   **Status**: Open
*   **Description**: File comments starting with `#` could contain inline configuration metadata (formatted as JSON or TOML block headers). We would like to parse these comments during run/qrels file ingestion and possibly use them to override or specify default evaluation settings (e.g. measure parameters) instead of relying solely on CLI flags.
*   **Options**:
    *   **Option A**: Store all comment lines in a structured metadata field within `RunData` and `QrelsData` structures during parsing, and defer JSON/TOML parsing to a separate post-parser config step.
    *   **Option B**: Skip comment storage for the MVP and implement this metadata parsing later once the basic evaluation pipeline is fully stable.

### 6. Double-Precision Floating-Point Compatibility (Rust vs C)
*   **Type**: Design
*   **Status**: Open
*   **Description**: C and Rust compile floating-point math differently depending on architecture instruction sets (e.g. x87 80-bit vs SSE2 64-bit, fast-math optimizations, register spill rounding). Microscopic numeric variations can lead to different values when printed to 4 decimal places. How do we ensure binary identical score outputs?
*   **Options**:
    *   **Option A**: Use standard IEEE 754 64-bit float calculations (`f64`) and evaluate differences during integration testing against the actual compiled C executable.
    *   **Option B**: Explicitly perform rounding or formatting normalization internally to match C's typical precision behavior.

### 7. Evaluation Corner Cases (Empty Runs, Zero-Relevance Topics)
*   **Type**: Design
*   **Status**: Open
*   **Description**: How should metrics handle topics with zero retrieved documents, zero judged documents, or zero relevant documents in the ground truth? Standard mathematics can result in division by zero.
*   **Options**:
    *   **Option A**: Define safe defaults (e.g. score of `0.0`) for any metric where the denominator is zero, to align completely with standard trec_eval behavior and prevent runtime panics.
    *   **Option B**: Throw distinct evaluation errors/warnings for these anomalies.

### 8. Printing and Formatting Types for Measures
*   **Type**: Design
*   **Status**: Open
*   **Description**: In C trec_eval, different measures are printed with different string format types. Standard floating-point measures (like `map`, `P_10`) are printed as 4-decimal-place floats, while count measures (like `num_q`, `num_ret`) are printed as integers.
*   **Options**:
    *   **Option A (Recommended)**: Define a `ValueFormat` enum (`Float` or `Integer`) in our codebase and require each `Measure` implementation to expose its desired output format via a trait method (e.g. `fn format(&self) -> ValueFormat`). The printing harness will cast and format the `f64` values accordingly.
    *   **Option B**: Keep formatting logic separate in a static registry map in the printing module rather than inside the `Measure` trait.

### 9. Representation and Printing of String-Valued Measures (e.g., `runid`, `relstring`)
*   **Type**: Design
*   **Status**: Open
*   **Description**: Some specialized measures do not produce numeric values, but instead output character strings (e.g. `runid` prints the run tag string, and `relstring` prints the sequence of relevance characters of the top N retrieved documents like `1--12-00-`).
*   **Options**:
    *   **Option A (Recommended)**: Define a `MetricValue` enum containing `Float(f64)`, `Integer(i64)`, and `Str(String)` variants, allowing the `Measure::calc` method to return structured values of any of these types. Expand the `ValueFormat` enum to include `Str` and `QuotedStr` variants to handle formatting (such as wrapping in single quotes for `relstring`).
    *   **Option B**: Cast string characters or ascii values into floats (like storing the pointer address or single character values as floats) and format them back to strings during printing. This option is highly un-idiomatic in Rust and prone to memory-safety risks.

### 10. Metric Compatibility with Ground-Truth Formats via Traits
*   **Type**: Design
*   **Status**: Open
*   **Description**: Some metrics only make sense for specific relevance judgment formats (e.g. standard measures like `map` require `qrels`, while preference measures like `prefs_simp` require `prefs` or `qrels_prefs` inputs). Running a metric on an incompatible format is nonsensical. How do we model and enforce this compatibility in Rust?
*   **Options**:
    *   **Option A (Recommended - Hybrid Static/Dynamic)**: Retain a single unified `Measure` trait, but introduce an `EvaluationType` enum (e.g., `Standard`, `Preferences`, `JudgmentGroups`) that each `Measure` must return via `fn eval_type(&self) -> EvaluationType`. The alignment engine compiles the parsed raw formats into an `EvalState` enum variant matching the file's format. Prior to execution, the harness does a type check to verify that all requested measures are compatible with the loaded `EvalState` format, aborting immediately with a clear diagnostic message if a mismatch is found.
    *   **Option B (Separate Traits)**: Split measures into entirely disjoint traits (e.g. `StandardMeasure` and `PrefsMeasure`). While statically checking compatibility, this prevents us from maintaining a simple heterogenous active-measure list (`Vec<Box<dyn Measure>>`) in the harness, requiring separate collection vectors and duplicate evaluation loop code.

### 11. Regression Testing Harness in Rust
*   **Type**: Design
*   **Status**: Open
*   **Description**: The C `trec_eval` includes a series of regression check files in its `test/` directory, verified using a `Makefile` target `quicktest`. To ensure that our Rust reimplementation produces identical results down to formatting and ties, we need a way to run these exact regression cases. How do we implement this inside the idiomatic Rust testing ecosystem?
*   **Options**:
    *   **Option A (Recommended)**: Create a `tests/regression.rs` integration test suite. Copy all `trec_eval/test/` data files into `te-rust/tests/test_data/`. During `cargo test`, compile the `te-rust` binary, execute it as a child process using `std::process::Command` against these copied inputs, capture the output stream, and compare it line-by-line against C's expected `out.*` files.
    *   **Option B**: Build a custom external bash script (`quicktest.sh`) to run the checks after compiling. While simple, this does not integrate with standard Rust `cargo test` workflows, making it harder for developers and CI pipelines to execute.

### 12. Avoiding Boundary Test Duplication via Shared Declarative Macros
*   **Type**: Design
*   **Status**: Open
*   **Description**: To check corner cases in metric calculation, we need to test boundary conditions (e.g. empty rankings, zero relevant documents, etc.) across dozens of different measures. Copying the mock-state setup code and assertions for each measure results in significant duplication. How do we avoid this?
*   **Options**:
    *   **Option A (Recommended)**: Create a shared test helper library containing unified state generators (like `make_mock_state`) and define a custom Rust macro `test_measure_boundaries!` to let metrics declare their expected boundary behaviors in a clean, unified, data-driven way.
    *   **Option B**: Let each measure define its own bespoke test assertions from scratch. While flexible, this leads to heavy maintenance overhead and high code repetition.

### 13. Enforcing Uniform Boundary Testing across all Metrics
*   **Type**: Design
*   **Status**: Open
*   **Description**: Even with helpers or macros, developers may still forget to write boundary tests for newly added metrics. To guarantee that standard corner cases (like empty rankings, zero-relevance topics, and unjudged-only pools) are tested *uniformly* across all current and future metrics, we need an automated enforcement mechanism.
*   **Options**:
    *   **Option A (Recommended - Automated Registry Loop)**: Leverage our central measure registry (e.g., `fn get_all_measures() -> Vec<Box<dyn Measure>>`) and write a single, unified test suite in `tests/uniform_boundaries.rs`. This suite automatically loops over every registered measure and subjects them to the entire boundary matrix. It asserts general mathematical invariants (e.g. standard precision/recall/MAP float measures must return exactly `0.0` under empty rankings or zero-relevance topics, and count measures must return `0` under empty rankings, and NO measure should ever panic or return NaN/infinite values).
    *   **Option B**: Rely solely on manual review or static checks to ensure each metric file has written its own separate test suite. This option is error-prone and doesn't scale as the number of metrics grows.









