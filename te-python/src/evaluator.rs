use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PySequence, PySet, PyString};
use te_rust::eval::evaluate_run;
use te_rust::io::QrelsData;
use te_rust::metrics::{registry::resolve_measures, EvalConfig};

use crate::conversions::{ingest_qrels, ingest_run, ingest_run_arrays, ColumnMapping};
use crate::result::EvalResult;

/// Core evaluation engine holding pre-indexed relevance judgments and evaluation configuration.
#[pyclass(subclass, name = "Evaluator")]
pub struct Evaluator {
    pub qrels: QrelsData,
    pub measure_names: Vec<String>,
    pub config: EvalConfig,
}


#[pymethods]
impl Evaluator {
    #[new]
    #[pyo3(signature = (
        qrels,
        measures = None,
        relevance_level = 1,
        complete_set_average = false,
        judged_docs_only = false,
        max_docs_per_topic = None,
        num_docs_in_coll = 0,
        global_gains = None,
        qid = None,
        docno = None,
        rel = None
    ))]
    pub fn new(
        qrels: &Bound<'_, PyAny>,
        measures: Option<&Bound<'_, PyAny>>,
        relevance_level: i64,
        complete_set_average: bool,
        judged_docs_only: bool,
        max_docs_per_topic: Option<usize>,
        num_docs_in_coll: usize,
        global_gains: Option<String>,
        qid: Option<String>,
        docno: Option<String>,
        rel: Option<String>,
    ) -> PyResult<Self> {
        let mapping = ColumnMapping {
            qid,
            docno,
            score_or_rel: rel,
        };
        let qrels_data = ingest_qrels(qrels, Some(&mapping))?;

        let measure_names = parse_measures_arg(measures)?;

        // Validate measure names by attempting resolution
        resolve_measures(&measure_names).map_err(|e| {
            PyValueError::new_err(format!("Invalid measure configuration: {}", e))
        })?;

        let parsed_gains = global_gains
            .as_ref()
            .map(|s| te_rust::metrics::common::GainsConfig::parse(s));

        let config = EvalConfig {
            query_flag: true,
            summary_flag: true,
            relevance_level,
            average_complete_flag: complete_set_average,
            judged_docs_only_flag: judged_docs_only,
            max_num_docs_per_topic: max_docs_per_topic.unwrap_or(usize::MAX),
            num_docs_in_coll,
            global_gains: parsed_gains,
        };

        Ok(Self {
            qrels: qrels_data,
            measure_names,
            config,
        })
    }

    /// Evaluate a system run against pre-indexed relevance judgments.
    #[pyo3(signature = (run, qid = None, docno = None, score = None))]
    pub fn evaluate(
        &self,
        py: Python<'_>,
        run: &Bound<'_, PyAny>,
        qid: Option<String>,
        docno: Option<String>,
        score: Option<String>,
    ) -> PyResult<EvalResult> {
        let mapping = ColumnMapping {
            qid,
            docno,
            score_or_rel: score,
        };
        let run_data = ingest_run(run, Some(&mapping))?;

        let active_measures = resolve_measures(&self.measure_names).map_err(|e| {
            PyValueError::new_err(format!("Measure resolution error: {}", e))
        })?;

        let qrels_ref = &self.qrels;
        let config_ref = &self.config;

        // Release GIL during computation
        let output = py.allow_threads(move || {
            evaluate_run(qrels_ref, &run_data, &active_measures, config_ref, None)
        });

        Ok(EvalResult::from_output(output))
    }

    /// Evaluate 1D contiguous arrays/sequences of query IDs, doc IDs, and scores.
    pub fn evaluate_arrays(
        &self,
        py: Python<'_>,
        query_ids: &Bound<'_, PyAny>,
        doc_ids: &Bound<'_, PyAny>,
        scores: &Bound<'_, PyAny>,
    ) -> PyResult<EvalResult> {
        let run_data = ingest_run_arrays(query_ids, doc_ids, scores)?;

        let active_measures = resolve_measures(&self.measure_names).map_err(|e| {
            PyValueError::new_err(format!("Measure resolution error: {}", e))
        })?;

        let qrels_ref = &self.qrels;
        let config_ref = &self.config;

        let output = py.allow_threads(move || {
            evaluate_run(qrels_ref, &run_data, &active_measures, config_ref, None)
        });

        Ok(EvalResult::from_output(output))
    }

}

fn parse_measures_arg(measures: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<String>> {
    let mut names = Vec::new();
    let Some(obj) = measures else {
        return Ok(vec!["official".to_string()]);
    };

    if let Ok(py_str) = obj.downcast::<PyString>() {
        let s = py_str.to_string_lossy().into_owned();
        names.push(s);
        return Ok(names);
    }

    if let Ok(set) = obj.downcast::<PySet>() {
        for item in set.iter() {
            let s = item.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected measure names in set to be strings")
            })?;
            names.push(s);
        }
        return Ok(names);
    }

    if let Ok(seq) = obj.downcast::<PySequence>() {
        let len = seq.len()?;
        for i in 0..len {
            let item = seq.get_item(i)?;
            let s = item.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected measure names in sequence to be strings")
            })?;
            names.push(s);
        }
        return Ok(names);
    }

    if let Ok(iter) = obj.iter() {
        for item_res in iter {
            let item = item_res?;
            let s = item.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected measure names in iterator to be strings")
            })?;
            names.push(s);
        }
        return Ok(names);
    }


    Err(PyTypeError::new_err(
        "Invalid measures argument: expected string, list of strings, set of strings, or None"
    ))
}
