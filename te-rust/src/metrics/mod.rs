pub mod common;
pub mod runid;
pub mod num_ret;
pub mod num_rel;
pub mod num_rel_ret;
pub mod map;
pub mod rprec;
pub mod recip_rank;
pub mod bpref;
pub mod precision;
pub mod ndcg_cut;
pub mod recall;
pub mod success;
pub mod avg_11pt;
pub mod utility;
pub mod relstring;

use crate::eval::QueryEvalState;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueFormat {
    /// Render as a double-precision float with exactly 4 decimal places (e.g. 0.2543).
    Float,
    /// Render as an integer (e.g. 4328).
    Integer,
    /// Render as a raw string (e.g. runid "my_runtag").
    Str,
    /// Render as a string wrapped in single quotes (e.g. relstring "'1-0-1'").
    QuotedStr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Str(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EvaluationType {
    Standard,
    Preferences,
}

#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub query_flag: bool,
    pub summary_flag: bool,
    pub relevance_level: i64,
    pub average_complete_flag: bool,
    pub judged_docs_only_flag: bool,
    pub max_num_docs_per_topic: usize,
    pub num_docs_in_coll: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            query_flag: false,
            summary_flag: true,
            relevance_level: 1,
            average_complete_flag: false,
            judged_docs_only_flag: false,
            max_num_docs_per_topic: usize::MAX,
            num_docs_in_coll: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalState {
    Standard(QueryEvalState),
}

pub trait Measure: Send + Sync {
    /// Unique root name of the measure (e.g. "map", "P").
    fn name(&self) -> &'static str;

    /// Short single-sentence summary of the measure (e.g. "Mean Average Precision").
    fn short_description(&self) -> &'static str;

    /// Detailed description/explanation of the measure.
    fn explanation(&self) -> &'static str;

    /// Printing format for the metric's values.
    fn format(&self) -> ValueFormat;

    /// The category of relevance judgments required by this measure.
    fn eval_type(&self) -> EvaluationType;

    /// Returns the exact sub-metric names to be calculated (e.g., `["P_5", "P_10", "P_15"]`).
    fn sub_metrics(&self) -> Vec<String>;

    /// Whether this measure should be reported in the summary averages at the end.
    fn is_summary_enabled(&self) -> bool {
        true
    }

    /// Returns the initial values for the running totals of this measure.
    fn initial_values(&self) -> Vec<MetricValue>;

    /// Calculate the score(s) for a single query.
    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue>;

    /// Accumulate a single query's scores into a running total.
    fn accumulate(&self, q_scores: &[MetricValue], running_totals: &mut [MetricValue]) {
        for (i, score) in q_scores.iter().enumerate() {
            match (score, &mut running_totals[i]) {
                (MetricValue::Float(s), MetricValue::Float(t)) => *t += s,
                (MetricValue::Integer(s), MetricValue::Integer(t)) => *t += s,
                _ => {} // Strings and non-matching types do not accumulate
            }
        }
    }

    /// Calculate the final summary score from the accumulated totals.
    fn average(&self, config: &EvalConfig, running_totals: &mut [MetricValue], num_queries_evaluated: usize, total_qrels_queries: usize) {
        let denominator = if config.average_complete_flag {
            total_qrels_queries
        } else {
            num_queries_evaluated
        };
        if denominator > 0 {
            for total in running_totals.iter_mut() {
                if let MetricValue::Float(t) = total {
                    *t /= denominator as f64;
                }
            }
        }
    }
}

/// Instantiates and returns the exact standard set of 'all_trec' measures.
pub fn get_measures_for_all_trec() -> Vec<Box<dyn Measure>> {
    vec![
        Box::new(runid::RunIdMeasure::new()),
        Box::new(num_ret::NumRetMeasure::new()),
        Box::new(num_rel::NumRelMeasure::new()),
        Box::new(num_rel_ret::NumRelRetMeasure::new()),
        Box::new(map::MapMeasure::new()),
        Box::new(rprec::RprecMeasure::new()),
        Box::new(recip_rank::RecipRankMeasure::new()),
        Box::new(precision::PrecisionCutMeasure::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])),
        Box::new(ndcg_cut::NdcgCutMeasure::new(vec![5, 10, 15, 20, 30, 100, 200, 500, 1000])),
        Box::new(bpref::BprefMeasure::new()),
    ]
}
