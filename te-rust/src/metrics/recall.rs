use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RecallCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl RecallCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("recall_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for RecallCutMeasure {
    fn name(&self) -> &'static str {
        "recall"
    }

    fn short_description(&self) -> &'static str {
        "Recall at cutoffs"
    }

    fn explanation(&self) -> &'static str {
        "Recall at cutoff ranks. Calculates the proportion of known relevant documents that have been retrieved by the system up to the specified cutoff ranks (e.g. top 5, 10, 15...)."
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
                let num_rel = q_state.num_rel;

                for &c in &self.cutoffs {
                    let rel_ret = super::common::count_relevant_retrieved_up_to(&q_state.results_rel_list, c, config);
                    let recall = if num_rel > 0 {
                        (rel_ret as f64) / (num_rel as f64)
                    } else {
                        0.0
                    };
                    results.push(MetricValue::Float(recall));
                }

                results
            }
        }
    }
}
