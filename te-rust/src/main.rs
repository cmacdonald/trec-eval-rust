use clap::Parser;
use std::collections::HashMap;
use std::process;

use te_rust::{eval, io, metrics};
use io::{parse_trec_qrels, parse_trec_run, QrelsQuery, RunQuery};
use eval::alignment::align_query;
use metrics::{EvalConfig, EvalState, MetricValue, ValueFormat};

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
    if help_measures {
        println!("{:<22} {:<13} {}", "Measure", "Status", "Description");
        println!("{}", "-".repeat(70));
        for spec in metrics::registry::registry() {
            if let Ok(m) = (spec.factory)("") {
                println!("{:<22} {:<13} {}", m.name(), spec.status.label(), m.short_description());
            }
        }
        println!("\nRequest measures with -m <name> (repeatable). See --help-measure <name> for details.");
        process::exit(0);
    }

    if let Some(target) = help_measure {
        for spec in metrics::registry::registry() {
            if spec.name.eq_ignore_ascii_case(target) {
                if let Ok(m) = (spec.factory)("") {
                    println!("{}", m.name());
                    if !spec.status.label().is_empty() {
                        println!("Status: {}", spec.status.label());
                    }
                    println!();
                    println!("{}", m.explanation());
                    println!();
                    // How to request it, and what parameters it accepts.
                    if spec.usage.is_empty() {
                        println!("Usage:  -m {}", m.name());
                    } else {
                        println!("Usage:  -m {}", spec.usage);
                    }
                    process::exit(0);
                }
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

    // 4. Resolve measures (groups + parameters) via the central registry.
    let requested_names: Vec<String> = if args.measures.is_empty() {
        vec!["official".to_string()]
    } else {
        args.measures.clone()
    };

    let active_measures = match metrics::registry::resolve_measures(&requested_names) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("te-rust: {}", e);
            process::exit(1);
        }
    };


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
