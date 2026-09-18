pub mod parsers;

pub use parsers::{
    parse_trec_qrels, parse_trec_run, IOError, ParseError, QrelsData, QrelsQuery, QrelsRecord,
    RunData, RunQuery, RunRecord,
};


