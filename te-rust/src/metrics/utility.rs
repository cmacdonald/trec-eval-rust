use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct UtilityMeasure {
    params: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl UtilityMeasure {
    pub fn new(params: Vec<f64>, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "utility".to_string()
        } else {
            format!("utility_{}", params_str)
        };
        Self { params, sub_metrics: vec![name] }
    }
}

impl Measure for UtilityMeasure {
    fn name(&self) -> &'static str {
        "utility"
    }

    fn short_description(&self) -> &'static str {
        "Set utility measure"
    }

    fn explanation(&self) -> &'static str {
        "Utility based on a contingency table of retrieved vs relevant docs. Computed as p1*a + p2*b + p3*c + p4*d, where a is retrieved-relevant, b is retrieved-nonrelevant, c is nonretrieved-relevant, d is nonretrieved-nonrelevant, and p1-p4 are weights (default: 1.0, -1.0, 0.0, 0.0)."
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
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let num_ret = q_state.results_rel_list.len();

                // Count total relevant retrieved
                let mut num_rel_ret = 0;
                for &rel in &q_state.results_rel_list {
                    if rel >= config.relevance_level {
                        num_rel_ret += 1;
                    }
                }

                let a = num_rel_ret as f64;
                let b = (num_ret - num_rel_ret) as f64;
                let c = (q_state.num_rel - num_rel_ret) as f64;
                
                // d = num_docs_in_coll + num_rel_ret - num_ret - num_rel
                let d = (config.num_docs_in_coll as i64 + num_rel_ret as i64 - num_ret as i64 - q_state.num_rel as i64) as f64;

                let score = self.params[0] * a +
                            self.params[1] * b +
                            self.params[2] * c +
                            self.params[3] * d;

                vec![MetricValue::Float(score)]
            }
        }
    }
}
