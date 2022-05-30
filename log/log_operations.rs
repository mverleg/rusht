use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group {
    name: String,
}

#[derive(Debug)]
pub enum Operation {
    RegexCount { regex: Regex },
    RegexCeaseSpan { regex: Regex },
}

#[derive(Debug)]
pub struct Stats {
    count: i64,
    min: i64,
    max: i64,
    sum: i64,
}

#[derive(Debug)]
pub enum Result {
    Count { name: String, counts: HashMap<Group, i64> },
    Stat { base_name: String, counts: HashMap<Group, Stats> },
}
