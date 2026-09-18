use std::collections::BTreeMap;
use std::path::Path;
use pyo3::exceptions::{PyFileNotFoundError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PySequence, PyString};
use te_rust::io::{parse_trec_qrels, parse_trec_run, QrelsData, QrelsQuery, QrelsRecord, RunData, RunQuery, RunRecord};

/// Extract a file path string from a Python object if it is a `str` or `os.PathLike`.
fn extract_path_string(obj: &Bound<'_, PyAny>) -> Option<String> {
    if let Ok(py_str) = obj.downcast::<PyString>() {
        return Some(py_str.to_string_lossy().into_owned());
    }
    if obj.hasattr("__fspath__").unwrap_or(false) {
        if let Ok(res) = obj.call_method0("__fspath__") {
            if let Ok(py_str) = res.downcast::<PyString>() {
                return Some(py_str.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Ingest relevance judgments (qrels) from a Python object (file path, dict, or iterable).
pub fn ingest_qrels(obj: &Bound<'_, PyAny>) -> PyResult<QrelsData> {
    // 1. File path / PathLike
    if let Some(path_str) = extract_path_string(obj) {
        let path = Path::new(&path_str);
        if path.is_file() {
            return parse_trec_qrels(path).map_err(|e| {
                PyValueError::new_err(format!("Failed to parse qrels file '{}': {}", path_str, e))
            });
        }
        return Err(PyFileNotFoundError::new_err(format!(
            "Qrels file not found: '{}'",
            path_str
        )));
    }

    // 2. Nested Dictionary: Dict[str, Dict[str, Union[int, float]]]
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut grouped: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
        for (qid_obj, docs_obj) in dict.iter() {
            let qid = qid_obj.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected query_id to be a string in qrels dict")
            })?;
            let docs_dict = docs_obj.downcast::<PyDict>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "Expected query '{}' value to be a Dict[doc_id, relevance] in qrels",
                    qid
                ))
            })?;

            let query_map = grouped.entry(qid).or_default();
            for (doc_obj, rel_obj) in docs_dict.iter() {
                let docno = doc_obj.extract::<String>().map_err(|_| {
                    PyTypeError::new_err("Expected doc_id to be a string in qrels dict")
                })?;
                let rel = rel_obj.extract::<i64>().map_err(|_| {
                    PyTypeError::new_err("Expected relevance judgment to be an integer in qrels dict")
                })?;
                query_map.insert(docno, rel);
            }
        }

        let queries = grouped
            .into_iter()
            .map(|(qid, doc_map)| QrelsQuery {
                qid,
                records: doc_map
                    .into_iter()
                    .map(|(docno, rel)| QrelsRecord { docno, rel })
                    .collect(),
            })
            .collect();

        return Ok(QrelsData {
            queries,
            comments: Vec::new(),
        });
    }

    // 3. Iterable of 3-tuples or objects with (query_id, doc_id, relevance)
    if let Ok(iter) = obj.iter() {
        return ingest_qrels_from_iter(iter);
    }


    Err(PyTypeError::new_err(
        "Invalid qrels input: expected file path, nested dict (Dict[str, Dict[str, int]]), or iterable of (qid, doc_id, rel) triples"
    ))
}

fn ingest_qrels_from_iter(iter: Bound<'_, PyIterator>) -> PyResult<QrelsData> {
    let mut grouped: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();

    for item_res in iter {
        let item = item_res?;
        let (qid, docno, rel) = extract_qrel_item(&item)?;
        grouped.entry(qid).or_default().insert(docno, rel);
    }

    let queries = grouped
        .into_iter()
        .map(|(qid, doc_map)| QrelsQuery {
            qid,
            records: doc_map
                .into_iter()
                .map(|(docno, rel)| QrelsRecord { docno, rel })
                .collect(),
        })
        .collect();

    Ok(QrelsData {
        queries,
        comments: Vec::new(),
    })
}

fn extract_qrel_item(item: &Bound<'_, PyAny>) -> PyResult<(String, String, i64)> {
    if let Ok(seq) = item.downcast::<PySequence>() {
        let len = seq.len()?;
        if len >= 3 {
            let qid = seq.get_item(0)?.extract::<String>()?;
            let docno = seq.get_item(1)?.extract::<String>()?;
            let rel = seq.get_item(2)?.extract::<i64>()?;
            return Ok((qid, docno, rel));
        }
    }

    // Check named attributes (e.g. ScoredDoc / Qrel / Dataclasses)
    let qid = if let Ok(v) = item.getattr("query_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("qid") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract query_id from qrel item (must be 3-tuple or have .query_id / .qid attribute)"
        ));
    };

    let docno = if let Ok(v) = item.getattr("doc_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("docno") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract doc_id from qrel item (must be 3-tuple or have .doc_id / .docno attribute)"
        ));
    };

    let rel = if let Ok(v) = item.getattr("relevance") {
        v.extract::<i64>()?
    } else if let Ok(v) = item.getattr("rel") {
        v.extract::<i64>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract relevance from qrel item (must be 3-tuple or have .relevance / .rel attribute)"
        ));
    };

    Ok((qid, docno, rel))
}

/// Ingest run results from a Python object (file path, dict, or iterable).
pub fn ingest_run(obj: &Bound<'_, PyAny>) -> PyResult<RunData> {
    // 1. File path / PathLike
    if let Some(path_str) = extract_path_string(obj) {
        let path = Path::new(&path_str);
        if path.is_file() {
            return parse_trec_run(path).map_err(|e| {
                PyValueError::new_err(format!("Failed to parse run file '{}': {}", path_str, e))
            });
        }
        return Err(PyFileNotFoundError::new_err(format!(
            "Run file not found: '{}'",
            path_str
        )));
    }

    // 2. Nested Dictionary: Dict[str, Dict[str, float]]
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();
        for (qid_obj, docs_obj) in dict.iter() {
            let qid = qid_obj.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected query_id to be a string in run dict")
            })?;
            let docs_dict = docs_obj.downcast::<PyDict>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "Expected query '{}' value to be a Dict[doc_id, score] in run",
                    qid
                ))
            })?;

            let records = grouped.entry(qid).or_default();
            for (doc_obj, sim_obj) in docs_dict.iter() {
                let docno = doc_obj.extract::<String>().map_err(|_| {
                    PyTypeError::new_err("Expected doc_id to be a string in run dict")
                })?;
                let sim = sim_obj.extract::<f64>().map_err(|_| {
                    PyTypeError::new_err("Expected score to be a float in run dict")
                })?;
                records.push(RunRecord { docno, sim });
            }
        }

        let queries = grouped
            .into_iter()
            .map(|(qid, records)| RunQuery { qid, records })
            .collect();

        return Ok(RunData {
            run_id: "trec_eval".to_string(),
            queries,
            comments: Vec::new(),
        });
    }

    // 3. Iterable of 3-tuples or objects with (query_id, doc_id, score)
    if let Ok(iter) = obj.iter() {
        return ingest_run_from_iter(iter);
    }


    Err(PyTypeError::new_err(
        "Invalid run input: expected file path, nested dict (Dict[str, Dict[str, float]]), or iterable of (qid, doc_id, score) triples"
    ))
}

fn ingest_run_from_iter(iter: Bound<'_, PyIterator>) -> PyResult<RunData> {
    let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();

    for item_res in iter {
        let item = item_res?;
        let (qid, docno, sim) = extract_run_item(&item)?;
        grouped.entry(qid).or_default().push(RunRecord { docno, sim });
    }

    let queries = grouped
        .into_iter()
        .map(|(qid, records)| RunQuery { qid, records })
        .collect();

    Ok(RunData {
        run_id: "trec_eval".to_string(),
        queries,
        comments: Vec::new(),
    })
}

fn extract_run_item(item: &Bound<'_, PyAny>) -> PyResult<(String, String, f64)> {
    if let Ok(seq) = item.downcast::<PySequence>() {
        let len = seq.len()?;
        if len >= 3 {
            let qid = seq.get_item(0)?.extract::<String>()?;
            let docno = seq.get_item(1)?.extract::<String>()?;
            let sim = seq.get_item(2)?.extract::<f64>()?;
            return Ok((qid, docno, sim));
        }
    }

    let qid = if let Ok(v) = item.getattr("query_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("qid") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract query_id from run item (must be 3-tuple or have .query_id / .qid attribute)"
        ));
    };

    let docno = if let Ok(v) = item.getattr("doc_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("docno") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract doc_id from run item (must be 3-tuple or have .doc_id / .docno attribute)"
        ));
    };

    let sim = if let Ok(v) = item.getattr("score") {
        v.extract::<f64>()?
    } else if let Ok(v) = item.getattr("sim") {
        v.extract::<f64>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract score from run item (must be 3-tuple or have .score / .sim attribute)"
        ));
    };

    Ok((qid, docno, sim))
}
