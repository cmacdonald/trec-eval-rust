use std::collections::HashMap;
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use te_rust::eval::EvaluationOutput;
use te_rust::metrics::MetricValue;


/// Rich evaluation result object providing mapping semantics and aggregation methods.
#[pyclass(name = "EvalResult")]
#[derive(Clone)]
pub struct EvalResult {
    /// Ordered list of topic IDs.
    pub qids: Vec<String>,
    /// Per-query metrics: qid -> (measure_name -> value).
    pub query_scores: HashMap<String, HashMap<String, MetricValue>>,
    /// Summary aggregate metrics: measure_name -> value.
    pub aggregate_scores: HashMap<String, MetricValue>,
    /// Number of queries evaluated.
    pub num_queries_evaluated: usize,
    /// Total queries in qrels.
    pub total_qrels_queries: usize,
}

impl EvalResult {
    pub fn from_output(output: EvaluationOutput) -> Self {
        let mut qids = Vec::with_capacity(output.query_results.len());
        let mut query_scores = HashMap::with_capacity(output.query_results.len());

        for q_eval in output.query_results {
            qids.push(q_eval.qid.clone());
            let mut scores = HashMap::with_capacity(q_eval.scores.len());
            for (name, val, _) in q_eval.scores {
                scores.insert(name, val);
            }
            query_scores.insert(q_eval.qid, scores);
        }

        let mut aggregate_scores = HashMap::with_capacity(output.summary_results.len());
        for s_eval in output.summary_results {
            aggregate_scores.insert(s_eval.name, s_eval.value);
        }

        Self {
            qids,
            query_scores,
            aggregate_scores,
            num_queries_evaluated: output.num_queries_evaluated,
            total_qrels_queries: output.total_qrels_queries,
        }
    }
}

fn metric_to_py(py: Python<'_>, val: &MetricValue) -> PyObject {
    match val {
        MetricValue::Float(f) => (*f).into_py(py),
        MetricValue::Integer(i) => (*i).into_py(py),
        MetricValue::Str(s) => s.as_str().into_py(py),
    }
}

#[pymethods]
impl EvalResult {
    /// Return the summary aggregate scores across all queries.
    pub fn aggregate(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);
        for (name, val) in &self.aggregate_scores {
            dict.set_item(name, metric_to_py(py, val))?;
        }
        Ok(dict.into_any().unbind())
    }

    /// Return the per-query scores as a nested dictionary: {qid: {measure: value}}.
    pub fn per_query(&self, py: Python<'_>) -> PyResult<PyObject> {
        let root = PyDict::new_bound(py);
        for qid in &self.qids {
            if let Some(scores) = self.query_scores.get(qid) {
                let q_dict = PyDict::new_bound(py);
                for (name, val) in scores {
                    q_dict.set_item(name, metric_to_py(py, val))?;
                }
                root.set_item(qid, q_dict)?;
            }
        }
        Ok(root.into_any().unbind())
    }

    /// Convert entire result to a nested dictionary for full dict compatibility.
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        self.per_query(py)
    }

    /// Number of evaluated queries.
    pub fn __len__(&self) -> usize {
        self.qids.len()
    }

    /// Indexing by query ID: returns dictionary of scores for that query.
    pub fn __getitem__(&self, py: Python<'_>, key: &str) -> PyResult<PyObject> {
        if let Some(scores) = self.query_scores.get(key) {
            let q_dict = PyDict::new_bound(py);
            for (name, val) in scores {
                q_dict.set_item(name, metric_to_py(py, val))?;
            }
            Ok(q_dict.into_any().unbind())
        } else {
            Err(PyKeyError::new_err(format!("Query '{}' not found in evaluation results", key)))
        }
    }


    /// Check if a query ID is present in the results.
    pub fn __contains__(&self, key: &str) -> bool {
        self.query_scores.contains_key(key)
    }

    /// Iterator over query IDs.
    pub fn __iter__(&self, py: Python<'_>) -> PyResult<PyObject> {
        let list = PyList::new_bound(py, &self.qids);
        let iter = list.call_method0("__iter__")?;
        Ok(iter.into_any().unbind())
    }

    /// List of evaluated query IDs.
    pub fn keys(&self) -> Vec<String> {
        self.qids.clone()
    }


    /// List of per-query metric dictionaries.
    pub fn values(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        let mut vals = Vec::with_capacity(self.qids.len());
        for qid in &self.qids {
            vals.push(self.__getitem__(py, qid)?);
        }
        Ok(vals)
    }

    /// List of (query_id, scores_dict) pairs.
    pub fn items(&self, py: Python<'_>) -> PyResult<Vec<(String, PyObject)>> {
        let mut items = Vec::with_capacity(self.qids.len());
        for qid in &self.qids {
            items.push((qid.clone(), self.__getitem__(py, qid)?));
        }
        Ok(items)
    }

    fn __repr__(&self) -> String {
        format!(
            "<EvalResult: {} queries evaluated, {} aggregate measures>",
            self.num_queries_evaluated,
            self.aggregate_scores.len()
        )
    }
}
