pub mod conversions;
pub mod evaluator;
pub mod result;

use pyo3::prelude::*;
use evaluator::Evaluator;
use result::EvalResult;

/// Python module definition for `_trec_eval`
#[pymodule]
fn _trec_eval(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "11.0.0.dev0")?;
    m.add_class::<Evaluator>()?;
    m.add_class::<EvalResult>()?;
    Ok(())
}

