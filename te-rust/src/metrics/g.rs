use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RelGain {
    pub rel_level: i64,
    pub gain: f64,
    pub num_at_level: usize,
}

#[derive(Debug, Clone)]
pub struct Gains {
    pub rel_gains: Vec<RelGain>,
    pub total_num_at_levels: usize,
}

impl Gains {
    pub fn setup(params_str: &str, rel_levels: &[usize]) -> Self {
        let mut custom_gains = HashMap::new();
        if !params_str.is_empty() {
            for pair_str in params_str.split(',') {
                let parts: Vec<&str> = pair_str.split('=').collect();
                if parts.len() == 2 {
                    if let (Ok(rel_level), Ok(gain)) = (parts[0].trim().parse::<i64>(), parts[1].trim().parse::<f64>()) {
                        custom_gains.insert(rel_level, gain);
                    }
                }
            }
        }

        let mut rel_gains = Vec::new();

        // 1. First add the custom gains specified in parameters
        for (&rel_level, &gain) in &custom_gains {
            let num_at_level = if rel_level >= 0 && (rel_level as usize) < rel_levels.len() {
                rel_levels[rel_level as usize]
            } else {
                0
            };
            rel_gains.push(RelGain {
                rel_level,
                gain,
                num_at_level,
            });
        }

        // 2. Then add all other relevance levels that were NOT specified in custom_gains
        for i in 0..rel_levels.len() {
            let rel_level = i as i64;
            if !custom_gains.contains_key(&rel_level) {
                rel_gains.push(RelGain {
                    rel_level,
                    gain: rel_level as f64,
                    num_at_level: rel_levels[i],
                });
            }
        }

        // 3. Sort by increasing gain value using total_cmp for determinism
        rel_gains.sort_by(|a, b| a.gain.total_cmp(&b.gain));

        let total_num_at_levels = rel_gains.iter().map(|g| g.num_at_level).sum();

        Gains {
            rel_gains,
            total_num_at_levels,
        }
    }

    pub fn get_gain(&self, rel_level: i64) -> f64 {
        for g in &self.rel_gains {
            if g.rel_level == rel_level {
                return g.gain;
            }
        }
        0.0
    }
}

pub struct GMeasure {
    params_str: String,
    sub_metrics: Vec<String>,
}

impl GMeasure {
    pub fn new(params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "G".to_string()
        } else {
            format!("G_{}", params_str)
        };
        Self {
            params_str: params_str.to_string(),
            sub_metrics: vec![name],
        }
    }
}

impl Measure for GMeasure {
    fn name(&self) -> &'static str {
        "G"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Gain"
    }

    fn explanation(&self) -> &'static str {
        "Experimental measure G, combining qualities of MAP and NDCG. G(doc) = gain(doc) / log2(2 + ideal_gain(i) - results_gain(i))."
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

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let gains = Gains::setup(&self.params_str, &q_state.rel_levels);

                let mut results_g = 0.0;
                let mut sum_results = 0.0;
                let mut sum_ideal = 0.0;
                let mut sum_cost = 0.0;
                let mut cur_level = gains.rel_gains.len() as i64 - 1;
                let mut ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
                let mut num_at_level = 0;
                let min_cost = 1.0;

                let mut i = 0;
                while i < q_state.num_ret && ideal_gain > 0.0 {
                    let results_gain = gains.get_gain(q_state.results_rel_list[i]);
                    sum_results += results_gain;

                    num_at_level += 1;
                    while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                        num_at_level = 1;
                        cur_level -= 1;
                        ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
                    }

                    if ideal_gain > 0.0 {
                        sum_ideal += ideal_gain;
                    }
                    if ideal_gain >= min_cost {
                        sum_cost += ideal_gain;
                    } else {
                        sum_cost += min_cost;
                    }

                    if results_gain != 0.0 {
                        results_g += results_gain / (2.0 + sum_cost - sum_results).log2();
                    }

                    i += 1;
                }

                while i < q_state.num_ret {
                    let results_gain = gains.get_gain(q_state.results_rel_list[i]);
                    sum_results += results_gain;
                    sum_cost += min_cost;

                    if results_gain != 0.0 {
                        results_g += results_gain / (2.0 + sum_cost - sum_results).log2();
                    }
                    i += 1;
                }

                while ideal_gain > 0.0 {
                    num_at_level += 1;
                    while cur_level >= 0 && num_at_level > gains.rel_gains[cur_level as usize].num_at_level {
                        num_at_level = 1;
                        cur_level -= 1;
                        ideal_gain = if cur_level >= 0 { gains.rel_gains[cur_level as usize].gain } else { 0.0 };
                    }
                    if ideal_gain > 0.0 {
                        sum_ideal += ideal_gain;
                    }
                }

                let score = if sum_ideal > 0.0 {
                    results_g / sum_ideal
                } else {
                    0.0
                };

                vec![MetricValue::Float(score)]
            }
        }
    }
}

impl Default for GMeasure {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_g_standard() {
        let measure = GMeasure::new("");
        let config = EvalConfig::default();
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
    }
}
