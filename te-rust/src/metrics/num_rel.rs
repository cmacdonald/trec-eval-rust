use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NumRelMeasure;

impl NumRelMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumRelMeasure {
    fn name(&self) -> &'static str {
        "num_rel"
    }

    fn short_description(&self) -> &'static str {
        "Total relevant"
    }

    fn explanation(&self) -> &'static str {
        "Total number of relevant documents. This is the count of documents that are judged relevant in the ground-truth relevance judgments (qrels) for the given topic."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_rel".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => vec![MetricValue::Integer(q_state.num_rel as i64)],
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for counts
    }
}
