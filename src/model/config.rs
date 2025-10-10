use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug)]
pub struct PageConfig {
    pub elements_per_page: usize,
    pub page_to_get: usize,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OutputToken<T> {
    Tab,
    Value(T),
    InArrow,
    OutArrow,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputLine<T> {
    pub line: Vec<OutputToken<T>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutputGraph<T> {
    pub parent_lines: Vec<OutputLine<T>>,
    pub child_lines: Vec<OutputLine<T>>,
    pub elements_data: HashMap<String, Value>,
}

pub enum RelationshipDirection {
    Parent,
    Child,
}

impl<T> OutputLine<T> {
    pub fn new() -> Self {
        Self { line: Vec::new() }
    }

    pub fn push(&mut self, token: OutputToken<T>) {
        self.line.push(token);
    }
}