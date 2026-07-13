use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NumRetMeasure;

impl NumRetMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumRetMeasure {
    fn name(&self) -> &'static str {
        "num_ret"
    }

    fn short_description(&self) -> &'static str {
        "Total retrieved"
    }

    fn explanation(&self) -> &'static str {
        "Total number of retrieved documents. This metric counts the absolute number of documents retrieved by the system across all queried topics."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_ret".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => vec![MetricValue::Integer(q_state.num_ret as i64)],
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for counts
    }
}
