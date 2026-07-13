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

// ==================== 9. 11pt Avg Measure ====================
pub struct Avg11PtMeasure {
    cutoffs: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl Avg11PtMeasure {
    pub fn new(cutoffs: Vec<f64>, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "11pt_avg".to_string()
        } else {
            format!("11pt_avg_{}", params_str)
        };
        Self { cutoffs, sub_metrics: vec![name] }
    }
}

impl Measure for Avg11PtMeasure {
    fn name(&self) -> &'static str {
        "11pt_avg"
    }

    fn short_description(&self) -> &'static str {
        "11pt Interpolated Average Precision"
    }

    fn explanation(&self) -> &'static str {
        "Interpolated Precision averaged over 11 recall points. Calculates interpolated precision scores at 11 standard recall levels (0.0, 0.1, 0.2, ..., 1.0) and averages them."
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
                let num_rel = q_state.num_rel;
                let num_ret = q_state.results_rel_list.len();

                if num_rel == 0 {
                    return vec![MetricValue::Float(0.0)];
                }

                // Count total relevant retrieved
                let mut num_rel_ret = 0;
                for &rel in &q_state.results_rel_list {
                    if rel >= config.relevance_level {
                        num_rel_ret += 1;
                    }
                }

                // Translate cutoff percentages to target relevant document counts
                let num_params = self.cutoffs.len();
                let mut cutoffs = vec![0i64; num_params];
                for i in 0..num_params {
                    cutoffs[i] = (self.cutoffs[i] * num_rel as f64).round() as i64;
                }

                let mut current_cut = num_params as i64 - 1;
                while current_cut >= 0 && cutoffs[current_cut as usize] > num_rel_ret as i64 {
                    current_cut -= 1;
                }

                let mut sum = 0.0;
                let mut int_precis = if num_ret > 0 {
                    (num_rel_ret as f64) / (num_ret as f64)
                } else {
                    0.0
                };
                let mut rel_so_far = num_rel_ret;

                for i in (1..=num_ret).rev() {
                    let precis = (rel_so_far as f64) / (i as f64);
                    if int_precis < precis {
                        int_precis = precis;
                    }
                    if q_state.results_rel_list[i - 1] >= config.relevance_level {
                        while current_cut >= 0 && rel_so_far as i64 == cutoffs[current_cut as usize] {
                            sum += int_precis;
                            current_cut -= 1;
                        }
                        rel_so_far -= 1;
                    }
                }

                while current_cut >= 0 {
                    sum += int_precis;
                    current_cut -= 1;
                }

                let final_val = sum / (num_params as f64);
                vec![MetricValue::Float(final_val)]
            }
        }
    }
}

// ==================== 10. Utility Measure ====================
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

// ==================== 11. Relstring Measure ====================
pub struct RelstringMeasure {
    len: usize,
    sub_metrics: Vec<String>,
}

impl RelstringMeasure {
    pub fn new(len: usize, params_str: &str) -> Self {
        let name = if params_str.is_empty() {
            "relstring".to_string()
        } else {
            format!("relstring_{}", params_str)
        };
        Self { len, sub_metrics: vec![name] }
    }
}

impl Measure for RelstringMeasure {
    fn name(&self) -> &'static str {
        "relstring"
    }

    fn short_description(&self) -> &'static str {
        "Relevance character visualizer string"
    }

    fn explanation(&self) -> &'static str {
        "Relevance string for the first N retrieved docs (default 10). Displays '0'-'9' for relevance grades, '>' for grades > 9, '-' for non-pool unjudged, '.' for pool unjudged, and '<' otherwise. Only reported for individual queries."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Str
    }

    fn is_summary_enabled(&self) -> bool {
        false
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Str(String::new())]
    }

    fn calc(&self, _config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        match state {
            EvalState::Standard(q_state) => {
                let limit = std::cmp::min(self.len, q_state.results_rel_list.len());
                let mut s = String::with_capacity(self.len);

                for i in 0..limit {
                    let rel = q_state.results_rel_list[i];
                    let c = if rel > 9 {
                        '>'
                    } else if rel >= 0 {
                        std::char::from_digit(rel as u32, 10).unwrap_or('<')
                    } else if rel == crate::eval::alignment::RELVALUE_NONPOOL {
                        '-'
                    } else if rel == crate::eval::alignment::RELVALUE_UNJUDGED {
                        '.'
                    } else {
                        '<'
                    };
                    s.push(c);
                }

                let quoted = format!("'{}'", s);
                vec![MetricValue::Str(quoted)]
            }
        }
    }
}

