use pyo3::prelude::*;

/// Python module definition for `_trec_eval`
#[pymodule]
fn _trec_eval(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "11.0.0.dev0")?;
    Ok(())
}
