pub(crate) type RuleResult = (u8, u8);
const CRITICAL_WEIGHT: u32 = 40;
const IMPORTANT_WEIGHT: u32 = 30;
const NORMAL_WEIGHT: u32 = 20;
const LOW_WEIGHT: u32 = 10;

// Score Calculation Formula
//
// The final Instrumentation Score ensures major issues significantly impact the score, and adheres to the 0-100 range.
//
// Let:
//
//     N be the total number of impact levels.
//     L i denote the i -th impact level, where i ∈ 1 , 2 , … , N .
//     W i be the weight assigned to the i -th impact level ( L i ).
//     P i be the number of rules passed, or succeeded, for impact level L i .
//     T i be the total number of rules for impact level L i .
//
// The Instrumentation Score is calculated as:
//
// Score = ∑ i = 1 N ( P i × W i ) ∑ i / 1 N ( T i × W i ) × 100
pub(crate) fn score(
    critical: RuleResult,
    important: RuleResult,
    normal: RuleResult,
    low: RuleResult,
) -> f64 {
    let impact_fn = |c_rules: u8, i_rules: u8, n_rules: u8, l_rules: u8| -> u32 {
        (c_rules as u32 * CRITICAL_WEIGHT)
            + (i_rules as u32 * IMPORTANT_WEIGHT)
            + (n_rules as u32 * NORMAL_WEIGHT)
            + (l_rules as u32 * LOW_WEIGHT)
    };
    ((impact_fn(critical.0, important.0, normal.0, low.0) as f64)
        / (impact_fn(critical.1, important.1, normal.1, low.1) as f64))
        * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_score() {
        let result = score((4, 8), (8, 10), (6, 8), (1, 5));
        assert_eq!(result, 63.85542168674698);
    }
}
