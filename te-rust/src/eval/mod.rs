pub mod alignment;
pub mod bootstrap;
pub mod engine;
pub mod significance;

pub use alignment::{align_query, QueryEvalState};
pub use bootstrap::{bootstrap_mean_ci, bootstrap_ci_with_aggregator, BootstrapConfig, ConfidenceInterval};
pub use engine::{evaluate_run, EvaluationOutput, QueryEvaluation, SummaryEvaluation};
pub use significance::{
    bootstrap_test, paired_t_test, permutation_test, SignificanceTestResult,
};


