# Remaining `trec_eval` Measures Roadmap

This document serves as a persistent checklist and implementation roadmap for the remaining non-preference `trec_eval` measures in `te-rust`.

---

## 1. Set-Based (Unranked) Measures
Set-based measures evaluate unranked retrieved sets (often using the `-M` parameter to restrict retrieved size).

- [ ] **`set_P`** (`m_set_P.c`): Set Precision: `num_relevant_retrieved / num_retrieved`.
- [ ] **`set_recall`** (`m_set_recall.c`): Set Recall: `num_relevant_retrieved / num_relevant`.
- [ ] **`set_relative_P`** (`m_set_rel_P.c`): Set Relative Precision (ratio of actual set precision to maximum possible set precision).
- [ ] **`set_map`** (`m_set_map.c`): Set Mean Average Precision.
- [ ] **`set_F`** (`m_set_F.c`): Set F-measure (weighted harmonic mean of set precision and set recall).

---

## 2. Cumulative Gain & NDCG Variants
Measures based on cumulated relevance grade gains, including non-standard NDCG versions.

- [ ] **`G`** (`m_G.c`): Cumulated Gain at various ranks (no discounting).
- [ ] **`binG`** (`m_binG.c`): Binary Cumulated Gain (gains are binary relevance grades).
- [ ] **`ndcg_rel`** (`m_ndcg_rel.c`): Graded NDCG relative (normalizes by ideal gain achievable by retrieved set size, rather than the entire corpus relevance pool).
- [ ] **`Rndcg`** (`m_Rndcg.c`): NDCG evaluated at rank $R$ (number of relevant documents).
- [ ] **`ndcg_p`** (`m_ndcg_p.c`): NDCG parameter-based / custom gain version.

---

## 3. Dynamic Cutoff & Precision Variants
Precision-at-cutoff variations.

- [ ] **`map_cut`** (`m_map_cut.c`): Mean Average Precision at specific cutoff rank thresholds.
- [ ] **`relative_P`** (`m_rel_P.c`): Relative Precision at cutoffs (precision at cutoff divided by max possible precision at that cutoff).
- [ ] **`Rprec_mult`** (`m_Rprec_mult.c`): R-precision multiple at cutoffs (evaluates at multiples of $R$, e.g., $2R$, $0.5R$).
- [ ] **`iprec_at_recall`** (`m_iprec_at_recall.c`): Interpolated Precision at 11 standard recall points (`0.00`, `0.10`, ..., `1.00`).

---

## 4. Geometric Mean & Inferred Relevance Measures
Measures useful for summarizing across highly skewed topic performance or handling incomplete judgments.

- [ ] **`gm_map`** (`m_gm_map.c`): Geometric Mean Average Precision.
- [ ] **`gm_bpref`** (`m_gm_bpref.c`): Geometric Mean bpref.
- [ ] **`infAP`** (`m_infap.c`): Inferred Average Precision (for incomplete judgment pools).

---

## 5. Pool & Judgment Statistics
Descriptive measures capturing pool coverage or unjudged proportions.

- [ ] **`unj`** (`m_unjudged.c`): Number of unjudged retrieved documents (at specific cutoffs, e.g., `unj_5`, `unj_10`).
- [ ] **`num_nonrel_judged_ret`** (`m_num_nonrel_judged_ret.c`): Total count of judged non-relevant documents retrieved.

---

## 6. Judgment Group (JG) / Agreement Measures
Agreement measures over multiple relevance judgment pools.

- [ ] **`map_avgjg`** (`m_map_avgjg.c`): MAP averaged across multiple judgment groups.
- [ ] **`P_avgjg`** (`m_P_avgjg.c`): Precision averaged across multiple judgment groups.
- [ ] **`Rprec_mult_avgjg`** (`m_Rprec_mult_avgjg.c`): R-precision multiples averaged across multiple judgment groups.

---

## 7. Modern & Specialized Measures
Other specialized metrics.

- [ ] **`rbp`** (`m_rbp.c`): Rank-Biased Precision.
- [ ] **`rbp_resid`** (`m_rbp.c`): Rank-Biased Precision Residual (representing the maximum possible error due to unjudged docs).
- [ ] **`yaap`** (`m_yaap.c`): Yet Another Average Precision.
