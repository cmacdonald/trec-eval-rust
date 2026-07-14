use clap::Parser;
use std::collections::HashMap;
use std::process;

mod io;
pub mod eval;
pub mod metrics;

use io::{parse_trec_qrels, parse_trec_run, QrelsQuery, RunQuery};
use eval::alignment::align_query;
use metrics::{EvalConfig, EvalState, Measure, MetricValue, ValueFormat};

#[derive(Parser, Debug)]
#[command(
    name = "te-rust",
    version = "0.1.0",
    about = "Rust clean-room implementation of trec_eval"
)]
struct Args {
    /// In addition to summary evaluation, print evaluation for each query/topic
    #[arg(short = 'q')]
    query_flag: bool,

    /// Print no summary averages/totals at the end
    #[arg(short = 'n')]
    no_summary_flag: bool,

    /// Minimum relevance level at which a document is considered relevant
    #[arg(short = 'l', default_value = "1")]
    relevance_level: i64,

    /// Average over the complete set of queries in qrels instead of the intersection
    #[arg(short = 'c')]
    complete_set_average: bool,

    /// Remove all unjudged documents from the retrieved set before evaluating
    #[arg(short = 'J')]
    judged_docs_only: bool,

    /// Max number of documents per topic to evaluate
    #[arg(short = 'M')]
    max_docs_per_topic: Option<usize>,

    /// Number of documents in the collection
    #[arg(short = 'N', default_value = "0")]
    num_docs_in_coll: usize,

    /// Calculate only the indicated measure(s)
    #[arg(short = 'm', value_name = "measure")]
    measures: Vec<String>,

    /// List all available measures with a short description and exit
    #[arg(long = "help-measures")]
    help_measures: bool,

    /// Print detailed explanation of the specified measure and exit
    #[arg(long = "help-measure", value_name = "name")]
    help_measure: Option<String>,

    /// Path to relevance judgments (qrels) file
    qrels_file: Option<String>,

    /// Path to run results file
    run_file: Option<String>,

    /// Global relevance-to-gain mapping (e.g. --global-gains 1=3.5,2=9.0)
    #[arg(long = "global-gains", value_name = "gains")]
    global_gains: Option<String>,
}

fn handle_help_flags(help_measures: bool, help_measure: Option<&str>) {
    let all_possible_measures: Vec<Box<dyn Measure>> = vec![
        Box::new(metrics::runid::RunIdMeasure::new()),
        Box::new(metrics::num_ret::NumRetMeasure::new()),
        Box::new(metrics::num_rel::NumRelMeasure::new()),
        Box::new(metrics::num_rel_ret::NumRelRetMeasure::new()),
        Box::new(metrics::map::MapMeasure::new()),
        Box::new(metrics::rprec::RprecMeasure::new()),
        Box::new(metrics::recip_rank::RecipRankMeasure::new()),
        Box::new(metrics::precision::PrecisionCutMeasure::new(vec![])),
        Box::new(metrics::ndcg_cut::NdcgCutMeasure::new(vec![])),
        Box::new(metrics::bpref::BprefMeasure::new()),
        Box::new(metrics::recall::RecallCutMeasure::new(vec![])),
        Box::new(metrics::success::SuccessCutMeasure::new(vec![])),
        Box::new(metrics::avg_11pt::Avg11PtMeasure::new(vec![], "")),
        Box::new(metrics::utility::UtilityMeasure::new(vec![0.0, 0.0, 0.0, 0.0], "")),
        Box::new(metrics::relstring::RelstringMeasure::new(0, "")),
        Box::new(metrics::map_cut::MapCutMeasure::new(vec![])),
        Box::new(metrics::relative_p::RelativePMeasure::new(vec![])),
        Box::new(metrics::rprec_mult::RprecMultMeasure::new(vec![])),
        Box::new(metrics::iprec_at_recall::IprecAtRecallMeasure::new(vec![])),
        Box::new(metrics::gm_map::GMMapMeasure::new()),
        Box::new(metrics::gm_bpref::GMBprefMeasure::new()),
        Box::new(metrics::infap::InfAPMeasure::new()),
        Box::new(metrics::unj::UnjMeasure::new(vec![])),
        Box::new(metrics::num_nonrel_judged_ret::NumNonrelJudgedRetMeasure::new()),
        Box::new(metrics::rbp::RbpMeasure::new(0.9, "")),
        Box::new(metrics::rbp_resid::RbpResidMeasure::new(0.9, "")),
        Box::new(metrics::yaap::YaapMeasure::new()),
    ];

    if help_measures {
        println!("{:<15}\t{}", "Measure", "Description");
        println!("--------------------------------------------------");
        for m in all_possible_measures {
            println!("{:<15}\t{}", m.name(), m.short_description());
        }
        process::exit(0);
    }

    if let Some(target) = help_measure {
        for m in all_possible_measures {
            if m.name().eq_ignore_ascii_case(target) {
                println!("{}", m.explanation());
                process::exit(0);
            }
        }
        eprintln!("te-rust: Unknown measure '{}'", target);
        process::exit(1);
    }
}

fn main() {
    let args = Args::parse();

    // 1. Handle help flags if present
    if args.help_measures || args.help_measure.is_some() {
        handle_help_flags(args.help_measures, args.help_measure.as_deref());
    }

    // 2. Validate positional files
    let qrels_filename = match args.qrels_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <QRELS_FILE>\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(1);
        }
    };

    let run_filename = match args.run_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(1);
        }
    };

    // 3. Ingest inputs
    let qrels_data = match parse_trec_qrels(&qrels_filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing qrels file '{}': {}", qrels_filename, e);
            process::exit(1);
        }
    };

    let run_data = match parse_trec_run(&run_filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error parsing run file '{}': {}", run_filename, e);
            process::exit(1);
        }
    };

    // 4. Resolve measures and predefined nicknames (groups)
    let mut requested_names = Vec::new();
    if args.measures.is_empty() {
        requested_names.push("official".to_string());
    } else {
        for m in &args.measures {
            requested_names.push(m.clone());
        }
    }

    let mut final_names = Vec::new();
    for name in &requested_names {
        match name.as_str() {
            "official" => {
                final_names.push("runid".to_string());
                final_names.push("num_ret".to_string());
                final_names.push("num_rel".to_string());
                final_names.push("num_rel_ret".to_string());
                final_names.push("map".to_string());
                final_names.push("Rprec".to_string());
                final_names.push("recip_rank".to_string());
                final_names.push("bpref".to_string());
                final_names.push("P".to_string());
            }
            "set" => {
                final_names.push("runid".to_string());
                final_names.push("num_ret".to_string());
                final_names.push("num_rel".to_string());
                final_names.push("num_rel_ret".to_string());
                final_names.push("set_relative_P".to_string());
                final_names.push("set_map".to_string());
                final_names.push("set_F".to_string());
            }
            "all_trec" => {
                final_names.push("runid".to_string());
                final_names.push("num_ret".to_string());
                final_names.push("num_rel".to_string());
                final_names.push("num_rel_ret".to_string());
                final_names.push("map".to_string());
                final_names.push("Rprec".to_string());
                final_names.push("recip_rank".to_string());
                final_names.push("bpref".to_string());
                final_names.push("P".to_string());
                final_names.push("ndcg_cut".to_string());
                final_names.push("ndcg".to_string());
                final_names.push("recall".to_string());
                final_names.push("success".to_string());
                final_names.push("11pt_avg".to_string());
                final_names.push("utility".to_string());
                final_names.push("relstring".to_string());
                final_names.push("set_relative_P".to_string());
                final_names.push("set_map".to_string());
                final_names.push("set_F".to_string());
                final_names.push("G".to_string());
            }
            other => {
                final_names.push(other.to_string());
            }
        }
    }

    let mut active_measures: Vec<Box<dyn Measure>> = Vec::new();
    for name in &final_names {
        let parts: Vec<&str> = name.splitn(2, '.').collect();
        let root = parts[0];
        let params_str = if parts.len() > 1 { parts[1] } else { "" };

        match root {
            "runid" => active_measures.push(Box::new(metrics::runid::RunIdMeasure::new())),
            "num_ret" => active_measures.push(Box::new(metrics::num_ret::NumRetMeasure::new())),
            "num_rel" => active_measures.push(Box::new(metrics::num_rel::NumRelMeasure::new())),
            "num_rel_ret" => active_measures.push(Box::new(metrics::num_rel_ret::NumRelRetMeasure::new())),
            "set_relative_P" => active_measures.push(Box::new(metrics::set_relative_p::SetRelativePMeasure::new())),
            "set_map" => active_measures.push(Box::new(metrics::set_map::SetMapMeasure::new())),
            "set_F" => {
                let beta = if params_str.is_empty() {
                    1.0
                } else {
                    match params_str.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("te-rust: Invalid float parameter '{}' in measure '{}'", params_str, name);
                            process::exit(1);
                        }
                    }
                };
                active_measures.push(Box::new(metrics::set_f::SetFMeasure::new(beta, params_str)));
            }
            "G" => active_measures.push(Box::new(metrics::g::GMeasure::new(params_str))),
            "map" => active_measures.push(Box::new(metrics::map::MapMeasure::new())),
            "Rprec" => active_measures.push(Box::new(metrics::rprec::RprecMeasure::new())),
            "recip_rank" => active_measures.push(Box::new(metrics::recip_rank::RecipRankMeasure::new())),
            "bpref" => active_measures.push(Box::new(metrics::bpref::BprefMeasure::new())),
            "P" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 15, 20, 30, 100, 200, 500, 1000]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::precision::PrecisionCutMeasure::new(cutoffs)));
            }
            "ndcg_cut" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 15, 20, 30, 100, 200, 500, 1000]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::ndcg_cut::NdcgCutMeasure::new(cutoffs)));
            }
            "ndcg" => {
                active_measures.push(Box::new(metrics::ndcg::NdcgMeasure::new(params_str)));
            }
            "recall" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 15, 20, 30, 100, 200, 500, 1000]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::recall::RecallCutMeasure::new(cutoffs)));
            }
            "success" => {
                let cutoffs = if params_str.is_empty() {
                    vec![1, 5, 10]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::success::SuccessCutMeasure::new(cutoffs)));
            }
            "11pt_avg" => {
                let cutoffs = if params_str.is_empty() {
                    vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<f64>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid float cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::avg_11pt::Avg11PtMeasure::new(cutoffs, params_str)));
            }
            "utility" => {
                let coeffs = if params_str.is_empty() {
                    vec![1.0, -1.0, 0.0, 0.0]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<f64>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid float coefficient '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    if list.len() != 4 {
                        eprintln!("te-rust: Improper number of coefficients (expected 4) in measure '{}'", name);
                        process::exit(1);
                    }
                    list
                };
                active_measures.push(Box::new(metrics::utility::UtilityMeasure::new(coeffs, params_str)));
            }
            "relstring" => {
                let len = if params_str.is_empty() {
                    10
                } else {
                    match params_str.trim().parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("te-rust: Invalid length '{}' in measure '{}'", params_str, name);
                            process::exit(1);
                        }
                    }
                };
                active_measures.push(Box::new(metrics::relstring::RelstringMeasure::new(len, params_str)));
            }
            "map_cut" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 15, 20, 30, 100, 200, 500, 1000]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::map_cut::MapCutMeasure::new(cutoffs)));
            }
            "relative_P" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 15, 20, 30, 100, 200, 500, 1000]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::relative_p::RelativePMeasure::new(cutoffs)));
            }
            "Rprec_mult" => {
                let cutoffs = if params_str.is_empty() {
                    vec![0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<f64>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid float cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::rprec_mult::RprecMultMeasure::new(cutoffs)));
            }
            "iprec_at_recall" => {
                let cutoffs = if params_str.is_empty() {
                    vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<f64>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid float cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::iprec_at_recall::IprecAtRecallMeasure::new(cutoffs)));
            }
            "gm_map" => {
                active_measures.push(Box::new(metrics::gm_map::GMMapMeasure::new()));
            }
            "gm_bpref" => {
                active_measures.push(Box::new(metrics::gm_bpref::GMBprefMeasure::new()));
            }
            "infAP" => {
                active_measures.push(Box::new(metrics::infap::InfAPMeasure::new()));
            }
            "unj" => {
                let cutoffs = if params_str.is_empty() {
                    vec![5, 10, 20]
                } else {
                    let mut list = Vec::new();
                    for s in params_str.split(',') {
                        match s.trim().parse::<usize>() {
                            Ok(v) => list.push(v),
                            Err(_) => {
                                eprintln!("te-rust: Invalid integer cutoff '{}' in measure '{}'", s, name);
                                process::exit(1);
                            }
                        }
                    }
                    list
                };
                active_measures.push(Box::new(metrics::unj::UnjMeasure::new(cutoffs)));
            }
            "num_nonrel_judged_ret" => {
                active_measures.push(Box::new(metrics::num_nonrel_judged_ret::NumNonrelJudgedRetMeasure::new()));
            }
            "rbp" => {
                let mut p = 0.9;
                if !params_str.is_empty() {
                    for part in params_str.split(',') {
                        let subparts: Vec<&str> = part.split('=').collect();
                        if subparts.len() == 2 && subparts[0].trim() == "p" {
                            match subparts[1].trim().parse::<f64>() {
                                Ok(v) => p = v,
                                Err(_) => {
                                    eprintln!("te-rust: Invalid float parameter '{}' in measure '{}'", params_str, name);
                                    process::exit(1);
                                }
                            }
                        }
                    }
                }
                active_measures.push(Box::new(metrics::rbp::RbpMeasure::new(p, params_str)));
            }
            "rbp_resid" => {
                let mut p = 0.9;
                if !params_str.is_empty() {
                    for part in params_str.split(',') {
                        let subparts: Vec<&str> = part.split('=').collect();
                        if subparts.len() == 2 && subparts[0].trim() == "p" {
                            match subparts[1].trim().parse::<f64>() {
                                Ok(v) => p = v,
                                Err(_) => {
                                    eprintln!("te-rust: Invalid float parameter '{}' in measure '{}'", params_str, name);
                                    process::exit(1);
                                }
                            }
                        }
                    }
                }
                active_measures.push(Box::new(metrics::rbp_resid::RbpResidMeasure::new(p, params_str)));
            }
            "yaap" => {
                active_measures.push(Box::new(metrics::yaap::YaapMeasure::new()));
            }
            other => {
                eprintln!("te-rust: Unknown measure '{}'", other);
                process::exit(1);
            }
        }
    }

    let mut qrels_by_qid: HashMap<String, &QrelsQuery> = HashMap::with_capacity(qrels_data.queries.len());
    for qrels_q in &qrels_data.queries {
        qrels_by_qid.insert(qrels_q.qid.clone(), qrels_q);
    }

    let mut run_by_qid: HashMap<String, &RunQuery> = HashMap::with_capacity(run_data.queries.len());
    for run_q in &run_data.queries {
        run_by_qid.insert(run_q.qid.clone(), run_q);
    }

    // 5. Collect topic IDs in alphabetical qid order (matching qrels_data sorting)
    let mut eval_qids = Vec::new();
    let mut num_queries_evaluated = 0;

    for qrels_q in &qrels_data.queries {
        let in_run = run_by_qid.contains_key(&qrels_q.qid);
        if in_run {
            num_queries_evaluated += 1;
        }
        if args.complete_set_average || in_run {
            eval_qids.push(qrels_q.qid.clone());
        }
    }

    // 6. Setup runtime config
    let global_gains = args.global_gains.as_ref().map(|s| crate::metrics::common::GainsConfig::parse(s));
    let config = EvalConfig {
        query_flag: args.query_flag,
        summary_flag: !args.no_summary_flag,
        relevance_level: args.relevance_level,
        average_complete_flag: args.complete_set_average,
        judged_docs_only_flag: args.judged_docs_only,
        max_num_docs_per_topic: args.max_docs_per_topic.unwrap_or(usize::MAX),
        num_docs_in_coll: args.num_docs_in_coll,
        global_gains,
    };

    // 7. Initialize running totals
    let mut running_totals: Vec<Vec<MetricValue>> = active_measures
        .iter()
        .map(|m| m.initial_values())
        .collect();

    // 8. Execution Loop: Query Evaluation
    for qid in &eval_qids {
        let qrels_q = qrels_by_qid.get(qid).unwrap(); // guaranteed to exist
        let run_q = run_by_qid.get(qid).copied();

        // Alignment stage
        let q_state = align_query(
            run_q,
            qrels_q,
            &run_data.run_id,
            config.relevance_level,
            config.max_num_docs_per_topic,
            config.judged_docs_only_flag,
        );

        let eval_state = EvalState::Standard(q_state);

        // Compute scores for each active measure
        for (m_idx, m) in active_measures.iter().enumerate() {
            let q_scores = m.calc(&config, &eval_state);

            if config.query_flag && m.is_query_enabled() {
                let sub_names = m.sub_metrics();
                for (sub_idx, val) in q_scores.iter().enumerate() {
                    let name = &sub_names[sub_idx];
                    print_metric_value(name, qid, val, m.format());
                }
            }

            m.accumulate(&q_scores, &mut running_totals[m_idx]);
        }
    }

    // 9. Summary averages / totals
    if config.summary_flag && num_queries_evaluated > 0 {
        let total_qrels_queries = qrels_data.queries.len();

        for (m_idx, m) in active_measures.iter().enumerate() {
            if !m.is_summary_enabled() {
                continue;
            }
            let mut totals = running_totals[m_idx].clone();
            m.average(&config, &mut totals, num_queries_evaluated, total_qrels_queries);

            let sub_names = m.sub_metrics();
            for (sub_idx, val) in totals.iter().enumerate() {
                let name = &sub_names[sub_idx];
                print_metric_value(name, "all", val, m.format());
            }
        }
    }
}

fn print_metric_value(name: &str, qid: &str, val: &MetricValue, format: ValueFormat) {
    match (val, format) {
        (MetricValue::Float(f), ValueFormat::Float) => {
            println!("{:<22}\t{}\t{:.4}", name, qid, f);
        }
        (MetricValue::Integer(i), ValueFormat::Integer) => {
            println!("{:<22}\t{}\t{}", name, qid, i);
        }
        (MetricValue::Str(s), ValueFormat::Str) => {
            println!("{:<22}\t{}\t{}", name, qid, s);
        }
        (MetricValue::Str(s), ValueFormat::QuotedStr) => {
            println!("{:<22}\t{}\t'{}'", name, qid, s);
        }
        _ => {
            // Fallback
            match val {
                MetricValue::Float(f) => println!("{:<22}\t{}\t{:.4}", name, qid, f),
                MetricValue::Integer(i) => println!("{:<22}\t{}\t{}", name, qid, i),
                MetricValue::Str(s) => println!("{:<22}\t{}\t{}", name, qid, s),
            }
        }
    }
}
