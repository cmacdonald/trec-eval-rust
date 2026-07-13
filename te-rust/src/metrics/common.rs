use crate::metrics::EvalConfig;

/// Helper function to count the number of relevant documents retrieved up to a certain rank limit.
pub(crate) fn count_relevant_retrieved_up_to(results_rel_list: &[i64], limit: usize, config: &EvalConfig) -> usize {
    let actual_limit = std::cmp::min(limit, results_rel_list.len());
    results_rel_list[..actual_limit]
        .iter()
        .filter(|&&rel| rel >= config.relevance_level)
        .count()
}
