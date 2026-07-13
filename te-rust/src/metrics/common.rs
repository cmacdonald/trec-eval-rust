use crate::metrics::EvalConfig;

/// Helper function to count the number of relevant documents retrieved up to a certain rank limit.
pub(crate) fn count_relevant_retrieved_up_to(results_rel_list: &[i64], limit: usize, config: &EvalConfig) -> usize {
    let actual_limit = std::cmp::min(limit, results_rel_list.len());
    results_rel_list[..actual_limit]
        .iter()
        .filter(|&&rel| rel >= config.relevance_level)
        .count()
}

#[cfg(test)]
pub(crate) fn make_mock_state(results_rel_list: Vec<i64>, num_rel: i64) -> crate::eval::alignment::QueryEvalState {
    let num_ret = results_rel_list.len();
    let num_rel_ret = results_rel_list.iter().filter(|&&r| r >= 1).count(); // assume relevance level cutoff >= 1
    
    let mut rel_levels = vec![0; 2];
    rel_levels[1] = num_rel as usize;
    rel_levels[0] = 100; // default count of non-relevant docs
    
    crate::eval::alignment::QueryEvalState {
        qid: "mock_topic".to_string(),
        run_id: "mock_run".to_string(),
        results_rel_list,
        num_ret,
        num_rel: num_rel as usize,
        num_rel_ret,
        num_nonpool: 0,
        num_unjudged_in_pool: 0,
        rel_levels,
    }
}
