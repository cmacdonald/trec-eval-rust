use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

// ==================== 1. RunId Measure ====================
pub struct RunIdMeasure;

impl RunIdMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for RunIdMeasure {
    fn name(&self) -> &'static str {
        "runid"
    }

    fn short_description(&self) -> &'static str {
        "Run identifier"
    }

    fn explanation(&self) -> &'static str {
        "Run identifier. This represents the unique string identifier or tag assigned to the retrieval system run under evaluation."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Str
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["runid".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Str("".to_string())]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => vec![MetricValue::Str(q_state.run_id.clone())],
        }
    }

    fn accumulate(&self, q_scores: &[MetricValue], running_totals: &mut [MetricValue]) {
        if let (MetricValue::Str(ref s), MetricValue::Str(ref mut t)) = (&q_scores[0], &mut running_totals[0]) {
            if t.is_empty() {
                *t = s.clone();
            }
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for runid
    }
}

// ==================== 2. NumRet Measure ====================
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

// ==================== 3. NumRel Measure ====================
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

// ==================== 4. NumRelRet Measure ====================
pub struct NumRelRetMeasure;

impl NumRelRetMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for NumRelRetMeasure {
    fn name(&self) -> &'static str {
        "num_rel_ret"
    }

    fn short_description(&self) -> &'static str {
        "Total relevant retrieved"
    }

    fn explanation(&self) -> &'static str {
        "Total number of relevant documents retrieved. This count measures the overlap between the documents retrieved by the system and the relevant documents judged in the qrels."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Integer
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["num_rel_ret".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Integer(0)]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => vec![MetricValue::Integer(q_state.num_rel_ret as i64)],
        }
    }

    fn average(&self, _config: &EvalConfig, _running_totals: &mut [MetricValue], _num_queries_evaluated: usize, _total_qrels_queries: usize) {
        // No-op for counts
    }
}

// ==================== 5. Map Measure ====================
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

// ==================== 6. Rprec Measure ====================
pub struct RprecMeasure;

impl RprecMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for RprecMeasure {
    fn name(&self) -> &'static str {
        "Rprec"
    }

    fn short_description(&self) -> &'static str {
        "R-Precision"
    }

    fn explanation(&self) -> &'static str {
        "R-Precision. The precision evaluated exactly at rank R, where R is the total number of known relevant documents for the query. This measure assesses system precision at a depth matching the size of the relevance pool."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["Rprec".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let r = q_state.num_rel;
                let mut rel_ret_at_r = 0;

                let limit = std::cmp::min(r, q_state.results_rel_list.len());
                for i in 0..limit {
                    if q_state.results_rel_list[i] >= config.relevance_level {
                        rel_ret_at_r += 1;
                    }
                }

                let score = if r > 0 {
                    (rel_ret_at_r as f64) / (r as f64)
                } else {
                    0.0
                };

                vec![MetricValue::Float(score)]
            }
        }
    }
}

// ==================== 7. RecipRank Measure ====================
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

// ==================== 8. Bpref Measure ====================
pub struct BprefMeasure;

impl BprefMeasure {
    pub fn new() -> Self {
        Self
    }
}

impl Measure for BprefMeasure {
    fn name(&self) -> &'static str {
        "bpref"
    }

    fn short_description(&self) -> &'static str {
        "Binary Preference"
    }

    fn explanation(&self) -> &'static str {
        "Binary Preference. This measure computes the preference of relevant documents over non-relevant documents. It is based on the relative ranks of judged documents, calculating how often a relevant document is retrieved before a non-relevant document."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        vec!["bpref".to_string()]
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let mut num_nonrel = 0;
                for j in 0..config.relevance_level as usize {
                    if j < q_state.rel_levels.len() {
                        num_nonrel += q_state.rel_levels[j];
                    }
                }

                let mut nonrel_so_far = 0;
                let mut bpref = 0.0;

                for &rel in &q_state.results_rel_list {
                    if rel == crate::eval::alignment::RELVALUE_NONPOOL {
                        continue;
                    }
                    if rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                        continue;
                    }

                    if rel >= 0 && rel < config.relevance_level {
                        nonrel_so_far += 1;
                    } else {
                        // Judged relevant document
                        if nonrel_so_far > 0 {
                            let min_nonrel_num_rel = std::cmp::min(nonrel_so_far, q_state.num_rel as i64) as f64;
                            let min_total_nonrel_num_rel = std::cmp::min(num_nonrel, q_state.num_rel) as f64;
                            bpref += 1.0 - (min_nonrel_num_rel / min_total_nonrel_num_rel);
                        } else {
                            bpref += 1.0;
                        }
                    }
                }

                if q_state.num_rel > 0 {
                    bpref /= q_state.num_rel as f64;
                } else {
                    bpref = 0.0;
                }

                vec![MetricValue::Float(bpref)]
            }
        }
    }
}
