use regex::Regex;

pub(crate) trait FullMatchableRegex {
    fn is_full_match(&self, haystack: &str) -> bool;
}

impl FullMatchableRegex for Regex {
    fn is_full_match(&self, haystack: &str) -> bool {
        self.find(haystack)
            .is_some_and(|m| m.start() == 0 && m.end() == haystack.len())
    }
}
