use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

static MAX_RESULT: usize = 10;


#[derive(Debug, Deserialize, JsonSchema)]
pub struct PageConfig {
    #[schemars(description = "Number of element contain in a single result or page")]
    pub elements_per_page: usize,
    #[schemars(description = "The page to retrieve. For example set to 1 to retrieve first page")]
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

impl Default for PageConfig {
    fn default() -> Self {
        Self { elements_per_page: MAX_RESULT, page_to_get: 1 as usize }
    }
}