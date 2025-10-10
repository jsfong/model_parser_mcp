use core::fmt;
use serde::{Deserialize, Serialize};
use std::error::Error;



#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ModelError {
    ModelGraphBuildingError(String),
    ModelNotFound(String, String),
    UnableToReadModel,
    InvalidInput(String),
    ParsingError(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::ModelGraphBuildingError(err) => {
                write!(f, "Error building model graph: {}", err)
            }
            ModelError::ModelNotFound(model, version) => {
                write!(f, "Model {} with version {} not found", model, version)
            }
            ModelError::InvalidInput(err) => write!(f, "Invalid input, {}",err),
            ModelError::ParsingError(err) => write!(f, "Unable to parse {}", err),
            ModelError::UnableToReadModel => write!(f, "Unable to read model"),
        }
    }
}

impl Error for ModelError {}

