use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

pub struct NdcgMeasure;

impl NdcgMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NdcgMeasure {
    fn name(&self) -> &'static str {
        "ndcg"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain. Computes NDCG over the entire retrieved ranking and entire ideal ranking."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["ndcg".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                // 1. Calculate Results DCG over the entire retrieved set
                let mut dcg = 0.0;
                for (i, &rel) in q_state.results_rel_list.iter().enumerate() {
                    if rel >= config.relevance_level {
                        dcg += (rel as f64) / ((i + 2) as f64).log2();
                    }
                }

                // 2. Calculate Ideal DCG over all relevant documents in descending order of grade
                let mut idcg = 0.0;
                let mut rank_idx = 0;
                // Iterate grades in descending order down to relevance_level
                for grade in (config.relevance_level as usize..q_state.rel_levels.len()).rev() {
                    let count = q_state.rel_levels[grade];
                    for _ in 0..count {
                        idcg += (grade as f64) / ((rank_idx + 2) as f64).log2();
                        rank_idx += 1;
                    }
                }

                let score = if idcg > 0.0 { dcg / idcg } else { 0.0 };
                vec![MetricValue::Float(score)]
            }
        }
    }
}

impl Default for NdcgMeasure {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_standard() {
        let measure = NdcgMeasure::new();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // dcg = 1.0 / log2(2) + 0.0 + 1.0 / log2(4) = 1.0 + 0.5 = 1.5.
        // idcg = 1.0 / log2(2) + 1.0 / log2(3) = 1.0 + 1.0 / 1.58496 = 1.63092975
        // ndcg = 1.5 / 1.63092975 = 0.91972079
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.919720791448).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_ndcg_empty_ranking() {
        let measure = NdcgMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![], 5);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0)]);
    }

    #[test]
    fn test_ndcg_zero_relevance() {
        let measure = NdcgMeasure::new();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 0);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.0)]);
    }
}
