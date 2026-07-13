use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

// ==================== 1. Precision at Cutoffs (P_cut) ====================
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

    fn explanation(&self) -> &'static str {
        "Precision at cutoffs"
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
                    let mut rel_ret = 0;
                    let limit = std::cmp::min(c, q_state.results_rel_list.len());

                    for i in 0..limit {
                        if q_state.results_rel_list[i] >= config.relevance_level {
                            rel_ret += 1;
                        }
                    }

                    let precision = (rel_ret as f64) / (c as f64);
                    results.push(MetricValue::Float(precision));
                }

                results
            }
        }
    }
}

// ==================== 2. Normalized Discounted Cumulative Gain at Cutoffs (ndcg_cut) ====================
pub struct NdcgCutMeasure {
    cutoffs: Vec<usize>,
    sub_metrics: Vec<String>,
}

impl NdcgCutMeasure {
    pub fn new(cutoffs: Vec<usize>) -> Self {
        let sub_metrics = cutoffs.iter().map(|c| format!("ndcg_cut_{}", c)).collect();
        Self { cutoffs, sub_metrics }
    }
}

impl Measure for NdcgCutMeasure {
    fn name(&self) -> &'static str {
        "ndcg_cut"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain at cutoffs"
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

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let num_params = self.cutoffs.len();
                let mut dcgs = vec![0.0; num_params];
                let mut idcgs = vec![0.0; num_params];

                // 1. Calculate DCG at each cutoff
                let mut cutoff_index = 0;
                let mut dcg_sum = 0.0;

                for i in 0..q_state.num_ret {
                    if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                        dcgs[cutoff_index] = dcg_sum;
                        cutoff_index += 1;
                        while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                            dcgs[cutoff_index] = dcg_sum;
                            cutoff_index += 1;
                        }
                        if cutoff_index >= num_params {
                            break;
                        }
                    }
                    let gain = q_state.results_rel_list[i];
                    if gain > 0 {
                        dcg_sum += (gain as f64) / ((i + 2) as f64).log2();
                    }
                }
                while cutoff_index < num_params {
                    dcgs[cutoff_index] = dcg_sum;
                    cutoff_index += 1;
                }

                // 2. Calculate Ideal DCG (IDCG) at each cutoff
                cutoff_index = 0;
                let mut cur_lvl = q_state.rel_levels.len() as i64 - 1;
                let mut lvl_count = 0;
                let mut ideal_dcg_sum = 0.0;

                let mut i = 0;
                loop {
                    lvl_count += 1;
                    while cur_lvl > 0 && lvl_count > q_state.rel_levels[cur_lvl as usize] {
                        cur_lvl -= 1;
                        lvl_count = 1;
                    }
                    if cur_lvl == 0 {
                        break;
                    }

                    if cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                        idcgs[cutoff_index] = ideal_dcg_sum;
                        cutoff_index += 1;
                        while cutoff_index < num_params && i == self.cutoffs[cutoff_index] {
                            idcgs[cutoff_index] = ideal_dcg_sum;
                            cutoff_index += 1;
                        }
                        if cutoff_index >= num_params {
                            break;
                        }
                    }

                    let gain = cur_lvl;
                    ideal_dcg_sum += (gain as f64) / ((i + 2) as f64).log2();
                    i += 1;
                }
                while cutoff_index < num_params {
                    idcgs[cutoff_index] = ideal_dcg_sum;
                    cutoff_index += 1;
                }

                // 3. Normalize DCG by IDCG
                let mut results = Vec::with_capacity(num_params);
                for k in 0..num_params {
                    let idcg = idcgs[k];
                    let dcg = dcgs[k];
                    let score = if idcg > 0.0 { dcg / idcg } else { 0.0 };
                    results.push(MetricValue::Float(score));
                }

                results
            }
        }
    }
}
