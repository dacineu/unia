use std::collections::HashSet;

pub struct SemanticMapper;

impl SemanticMapper {
    /// Computes a similarity score between two strings based on token overlap.
    /// Returns a value between 0.0 (no overlap) and 1.0 (perfect overlap).
    pub fn compute_score(intent: &str, target: &str) -> f64 {
        let intent_tokens = Self::tokenize(intent);
        let target_tokens = Self::tokenize(target);

        if intent_tokens.is_empty() || target_tokens.is_empty() {
            return 0.0;
        }

        let intersection = intent_tokens.intersection(&target_tokens).count();
        let union = intent_tokens.union(&target_tokens).count();

        intersection as f64 / union as f64
    }

    fn tokenize(text: &str) -> HashSet<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
