use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct MapMeasure;

impl MapMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for MapMeasure {
    fn name(&self) -> &'static str {
        "map"
    }

    fn short_description(&self) -> &'static str {
        "Mean Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Mean Average Precision. Calculates the average of precision scores evaluated at each rank where a relevant document is retrieved. For documents not retrieved, precision is defined as 0. Over queried topics, MAP represents the mean of average precisions."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["map".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let mut rel_so_far = 0;
                let mut sum = 0.0;

                for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                    if rel >= config.relevance_level {
                        rel_so_far += 1;
                        sum += (rel_so_far as f64) / (i + 1) as f64;
                    }
                }

                let ap = if q_state.num_rel > 0 {
                    sum / (q_state.num_rel as f64)
                } else {
                    0.0
                };

                vec![MetricValue::Float(ap)]
            }
        }
    }
}
