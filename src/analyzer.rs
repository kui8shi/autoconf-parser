//! Provides the higher level on-time analyses to improve parsing
use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::Searcher,
    searcher::sinks::UTF8,
};

use super::m4_macro::M4Type;

/// doc
pub trait Analyzer {
    /// doc
    fn search_macro_definition(name: &str) -> Option<(Vec<M4Type>, M4Type, Option<(usize, usize)>)>;
}

/// doc
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct GrepAnalyzer {
}

impl Analyzer for GrepAnalyzer {
    fn search_macro_definition(name: &str) -> Option<(Vec<M4Type>, M4Type, Option<(usize, usize)>)>{
        None
    }
}
