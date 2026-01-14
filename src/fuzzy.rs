use nucleo::{
    pattern::{CaseMatching, Normalization, Pattern},
    Matcher, Config,
};

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// Filter items by fuzzy matching against a query.
    /// Returns indices of matching items, sorted by score (best first).
    pub fn filter(&mut self, items: &[String], query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }

        let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

        let mut matches: Vec<(usize, u32)> = items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                let mut buf = Vec::new();
                let haystack = nucleo::Utf32Str::new(item, &mut buf);
                pattern
                    .score(haystack, &mut self.matcher)
                    .map(|score| (idx, score))
            })
            .collect();

        // Sort by score descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        matches.into_iter().map(|(idx, _)| idx).collect()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}
