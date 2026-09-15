//! Central registry of measures, the single source of truth for measure names,
//! parameter parsing, and construction. Analogous to C `trec_eval`'s
//! `te_trec_measures[]` table in `measures.c`.
//!
//! Each measure is described by a [`MeasureSpec`] carrying its name, status, and
//! a stateless factory function that builds an instance from the parameter
//! string that follows the `.` in a measure argument (e.g. the `10,20` in
//! `P.10,20`). Groups such as `official` and `all_trec` are name lists resolved
//! against this table.

use crate::metrics::common::{
    parse_float_cutoffs, parse_int_cutoffs, parse_key_values, MeasureParseError,
};
use crate::metrics::{self, Measure};

/// Development/support status of a measure (ISSUES #24).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeasureStatus {
    /// Standard, supported measure.
    Production,
    /// Experimental measure; use with care.
    Experimental,
    /// Obsolete measure kept for compatibility.
    Obsolete,
}

/// Factory building a measure instance from its parameter string.
type MeasureFactory = fn(&str) -> Result<Box<dyn Measure>, MeasureParseError>;

/// Static description of a single measure root.
pub struct MeasureSpec {
    pub name: &'static str,
    pub status: MeasureStatus,
    pub factory: MeasureFactory,
}

// Default cutoff/parameter sets, matching the previous main.rs behavior exactly.
const DEFAULT_RANK_CUTOFFS: &[usize] = &[5, 10, 15, 20, 30, 100, 200, 500, 1000];
const DEFAULT_SUCCESS_CUTOFFS: &[usize] = &[1, 5, 10];
const DEFAULT_UNJ_CUTOFFS: &[usize] = &[5, 10, 20];
const DEFAULT_RECALL_LEVELS: &[f64] = &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
const DEFAULT_RPREC_MULT: &[f64] = &[0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, 1.8, 2.0];
const DEFAULT_UTILITY_COEFFS: &[f64] = &[1.0, -1.0, 0.0, 0.0];

/// Extract the RBP-style `p=<float>` parameter, defaulting to 0.9.
fn parse_rbp_p(params: &str) -> Result<f64, MeasureParseError> {
    let mut p = 0.9;
    for (k, v) in parse_key_values(params)? {
        if k == "p" {
            p = v
                .parse::<f64>()
                .map_err(|_| MeasureParseError::new(format!("invalid float parameter '{}'", v)))?;
        }
    }
    Ok(p)
}

/// The full measure registry table.
pub fn registry() -> &'static [MeasureSpec] {
    use MeasureStatus::*;
    &[
        MeasureSpec { name: "runid", status: Production, factory: |_| Ok(Box::new(metrics::runid::RunIdMeasure::new())) },
        MeasureSpec { name: "num_ret", status: Production, factory: |_| Ok(Box::new(metrics::num_ret::NumRetMeasure::new())) },
        MeasureSpec { name: "num_rel", status: Production, factory: |_| Ok(Box::new(metrics::num_rel::NumRelMeasure::new())) },
        MeasureSpec { name: "num_rel_ret", status: Production, factory: |_| Ok(Box::new(metrics::num_rel_ret::NumRelRetMeasure::new())) },
        MeasureSpec { name: "num_nonrel_judged_ret", status: Production, factory: |_| Ok(Box::new(metrics::num_nonrel_judged_ret::NumNonrelJudgedRetMeasure::new())) },
        MeasureSpec { name: "map", status: Production, factory: |_| Ok(Box::new(metrics::map::MapMeasure::new())) },
        MeasureSpec { name: "gm_map", status: Production, factory: |_| Ok(Box::new(metrics::gm_map::GMMapMeasure::new())) },
        MeasureSpec { name: "Rprec", status: Production, factory: |_| Ok(Box::new(metrics::rprec::RprecMeasure::new())) },
        MeasureSpec { name: "recip_rank", status: Production, factory: |_| Ok(Box::new(metrics::recip_rank::RecipRankMeasure::new())) },
        MeasureSpec { name: "bpref", status: Production, factory: |_| Ok(Box::new(metrics::bpref::BprefMeasure::new())) },
        MeasureSpec { name: "gm_bpref", status: Production, factory: |_| Ok(Box::new(metrics::gm_bpref::GMBprefMeasure::new())) },
        MeasureSpec { name: "infAP", status: Production, factory: |_| Ok(Box::new(metrics::infap::InfAPMeasure::new())) },
        MeasureSpec {
            name: "P",
            status: Production,
            factory: |p| Ok(Box::new(metrics::precision::PrecisionCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "recall",
            status: Production,
            factory: |p| Ok(Box::new(metrics::recall::RecallCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "ndcg_cut",
            status: Production,
            factory: |p| Ok(Box::new(metrics::ndcg_cut::NdcgCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "map_cut",
            status: Production,
            factory: |p| Ok(Box::new(metrics::map_cut::MapCutMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "relative_P",
            status: Production,
            factory: |p| Ok(Box::new(metrics::relative_p::RelativePMeasure::new(parse_int_cutoffs(p, DEFAULT_RANK_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "success",
            status: Production,
            factory: |p| Ok(Box::new(metrics::success::SuccessCutMeasure::new(parse_int_cutoffs(p, DEFAULT_SUCCESS_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "unj",
            status: Production,
            factory: |p| Ok(Box::new(metrics::unj::UnjMeasure::new(parse_int_cutoffs(p, DEFAULT_UNJ_CUTOFFS)?))),
        },
        MeasureSpec {
            name: "11pt_avg",
            status: Production,
            factory: |p| Ok(Box::new(metrics::avg_11pt::Avg11PtMeasure::new(parse_float_cutoffs(p, DEFAULT_RECALL_LEVELS)?, p))),
        },
        MeasureSpec {
            name: "iprec_at_recall",
            status: Production,
            factory: |p| Ok(Box::new(metrics::iprec_at_recall::IprecAtRecallMeasure::new(parse_float_cutoffs(p, DEFAULT_RECALL_LEVELS)?))),
        },
        MeasureSpec {
            name: "Rprec_mult",
            status: Production,
            factory: |p| Ok(Box::new(metrics::rprec_mult::RprecMultMeasure::new(parse_float_cutoffs(p, DEFAULT_RPREC_MULT)?))),
        },
        MeasureSpec {
            name: "utility",
            status: Production,
            factory: |p| {
                let coeffs = parse_float_cutoffs(p, DEFAULT_UTILITY_COEFFS)?;
                if coeffs.len() != 4 {
                    return Err(MeasureParseError::new("utility requires exactly 4 coefficients"));
                }
                Ok(Box::new(metrics::utility::UtilityMeasure::new(coeffs, p)))
            },
        },
        MeasureSpec {
            name: "set_F",
            status: Production,
            factory: |p| {
                let beta = if p.is_empty() {
                    1.0
                } else {
                    p.parse::<f64>().map_err(|_| MeasureParseError::new(format!("invalid float parameter '{}'", p)))?
                };
                Ok(Box::new(metrics::set_f::SetFMeasure::new(beta, p)))
            },
        },
        MeasureSpec {
            name: "relstring",
            status: Production,
            factory: |p| {
                let len = if p.is_empty() {
                    10
                } else {
                    p.trim().parse::<usize>().map_err(|_| MeasureParseError::new(format!("invalid length '{}'", p)))?
                };
                Ok(Box::new(metrics::relstring::RelstringMeasure::new(len, p)))
            },
        },
        MeasureSpec { name: "set_relative_P", status: Production, factory: |_| Ok(Box::new(metrics::set_relative_p::SetRelativePMeasure::new())) },
        MeasureSpec { name: "set_map", status: Production, factory: |_| Ok(Box::new(metrics::set_map::SetMapMeasure::new())) },
        MeasureSpec { name: "G", status: Production, factory: |p| Ok(Box::new(metrics::g::GMeasure::new(p))) },
        MeasureSpec { name: "ndcg", status: Production, factory: |p| Ok(Box::new(metrics::ndcg::NdcgMeasure::new(p))) },
        MeasureSpec { name: "ndcg_rel", status: Production, factory: |p| Ok(Box::new(metrics::ndcg_rel::NdcgRelMeasure::new(p))) },
        MeasureSpec { name: "Rndcg", status: Production, factory: |p| Ok(Box::new(metrics::rndcg::RndcgMeasure::new(p))) },
        MeasureSpec { name: "ndcg_p", status: Production, factory: |p| Ok(Box::new(metrics::ndcg_p::NdcgPMeasure::new(p))) },
        MeasureSpec { name: "rbp", status: Experimental, factory: |p| Ok(Box::new(metrics::rbp::RbpMeasure::new(parse_rbp_p(p)?, p))) },
        MeasureSpec { name: "rbp_resid", status: Experimental, factory: |p| Ok(Box::new(metrics::rbp_resid::RbpResidMeasure::new(parse_rbp_p(p)?, p))) },
        MeasureSpec { name: "yaap", status: Experimental, factory: |_| Ok(Box::new(metrics::yaap::YaapMeasure::new())) },
        MeasureSpec { name: "binG", status: Experimental, factory: |_| Ok(Box::new(metrics::bin_g::BinGMeasure::new())) },
    ]
}

/// Look up a measure spec by its root name.
pub fn find_spec(name: &str) -> Option<&'static MeasureSpec> {
    registry().iter().find(|s| s.name == name)
}

/// Expand a predefined group name into its list of member measure names.
/// Returns `None` if the name is not a known group.
pub fn expand_group(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "official" => Some(&["runid", "num_ret", "num_rel", "num_rel_ret", "map", "Rprec", "recip_rank", "bpref", "P"]),
        "set" => Some(&["runid", "num_ret", "num_rel", "num_rel_ret", "set_relative_P", "set_map", "set_F"]),
        "all_trec" => Some(&[
            "runid", "num_ret", "num_rel", "num_rel_ret", "map", "Rprec", "recip_rank", "bpref", "P",
            "ndcg_cut", "ndcg", "recall", "success", "11pt_avg", "utility", "relstring",
            "set_relative_P", "set_map", "set_F", "G",
        ]),
        _ => None,
    }
}

/// Resolve a list of requested measure arguments into constructed measures.
///
/// Each request is either a group name (expanded via [`expand_group`]) or a
/// `root[.params]` measure specification. Parameters after the first `.` are
/// passed to the measure's factory. Errors carry the offending argument.
pub fn resolve_measures(requested: &[String]) -> Result<Vec<Box<dyn Measure>>, MeasureParseError> {
    // 1. Expand any groups into concrete measure argument strings.
    let mut expanded: Vec<String> = Vec::new();
    for req in requested {
        match expand_group(req) {
            Some(members) => expanded.extend(members.iter().map(|m| m.to_string())),
            None => expanded.push(req.clone()),
        }
    }

    // 2. Build each measure from its root and parameter string.
    let mut measures: Vec<Box<dyn Measure>> = Vec::with_capacity(expanded.len());
    for arg in &expanded {
        let (root, params) = match arg.split_once('.') {
            Some((r, p)) => (r, p),
            None => (arg.as_str(), ""),
        };
        let spec = find_spec(root)
            .ok_or_else(|| MeasureParseError::new(format!("unknown measure '{}'", root)))?;
        let measure = (spec.factory)(params)
            .map_err(|e| MeasureParseError::new(format!("measure '{}': {}", arg, e)))?;
        measures.push(measure);
    }
    Ok(measures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_factory_builds_with_defaults() {
        for spec in registry() {
            let m = (spec.factory)("").unwrap_or_else(|e| panic!("{} failed: {}", spec.name, e));
            assert_eq!(m.name(), spec.name, "spec name must match measure name");
        }
    }

    #[test]
    fn official_group_expands() {
        let ms = resolve_measures(&["official".to_string()]).unwrap();
        let names: Vec<_> = ms.iter().map(|m| m.name()).collect();
        assert_eq!(names, vec!["runid", "num_ret", "num_rel", "num_rel_ret", "map", "Rprec", "recip_rank", "bpref", "P"]);
    }

    #[test]
    fn parameterized_measure_parses() {
        let ms = resolve_measures(&["P.10,20".to_string()]).unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].sub_metrics(), vec!["P_10".to_string(), "P_20".to_string()]);
    }

    #[test]
    fn unknown_measure_errors() {
        assert!(resolve_measures(&["nope".to_string()]).is_err());
    }

    #[test]
    fn bad_param_errors_with_context() {
        match resolve_measures(&["P.x".to_string()]) {
            Err(e) => assert!(e.message.contains("P.x"), "error should name the argument: {}", e.message),
            Ok(_) => panic!("expected an error"),
        }
    }

    #[test]
    fn utility_requires_four_coeffs() {
        assert!(resolve_measures(&["utility.1.0,2.0".to_string()]).is_err());
    }
}
