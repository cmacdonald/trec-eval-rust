use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct RecipRankMeasure;

impl RecipRankMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for RecipRankMeasure {
    fn name(&self) -> &'static str {
        "recip_rank"
    }

    fn short_description(&self) -> &'static str {
        "Reciprocal Rank"
    }

    fn explanation(&self) -> &'static str {
        "Reciprocal Rank. Defined as 1/K, where K is the rank of the first relevant document retrieved by the system. If no relevant documents are retrieved, the score is 0.0."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["recip_rank".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let mut score = 0.0;
                for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                    if rel >= config.relevance_level {
                        score = 1.0 / (i + 1) as f64;
                        break;
                    }
                }
                vec![MetricValue::Float(score)]
            }
        }
    }
}
