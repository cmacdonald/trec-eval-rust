use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct PCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl PCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("P_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for PCutMeasure {
    fn name(&self) -> &'static str {
        "P"
    }

    fn short_description(&self) -> &'static str {
        "Precision at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Precision at cutoff ranks. Calculates the proportion of retrieved documents that are relevant, evaluated at specific rank thresholds (e.g. top 5, 10, 15... retrieved documents). It measures retrieval accuracy at specified system output depths."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0); self.cutoffs.len()]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let mut results = Vec::with_capacity(self.cutoffs.len());

                for &c in &self.cutoffs {
                    let rel_ret = super::common::count_relevant_retrieved_up_to(&q_state.results_rel_list, c, config);
                    let precision = (rel_ret as f64) / (c as f64);
                    results.push(MetricValue::Float(precision));
                }

                results
            }
        }
    }
}
