# trec-eval Python Package

High-performance Python bindings for `trec_eval`, powered by `te-rust`.

## Installation

```bash
pip install trec-eval
```

## Dependencies

`trec-eval` has **zero mandatory dependencies**. Optional integrations are available:

- `pip install "trec-eval[numpy]"`: Zero-copy NumPy buffer ingestion
- `pip install "trec-eval[pandas]"`: DataFrame input/output support
- `pip install "trec-eval[stats]"`: Statistical tests & corrections
- `pip install "trec-eval[all]"`: All optional dependencies (Pandas, Polars, NumPy, SciPy, PyArrow)
