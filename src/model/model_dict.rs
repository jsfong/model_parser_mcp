use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

use super::cubs_model::ModelData;
use crate::model::cubs_model::{CusObject, Element, ModelVersionNumber, Relationship};

#[derive(Debug, Serialize)]
pub struct ModelDictionary {
    pub model_id: String,
    pub version: u32,
    pub model_stats: ModelStats,
    pub model_versions: Vec<ModelVersionNumber>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelStats {
    pub elements_stats: Option<CubsObjectReport>,
    pub relationships_stats: Option<CubsObjectReport>,
    pub version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CubsObjectReport {
    pub all_count: u32,
    pub by_type: ElementCounts,
    pub by_nature: ElementCounts,
}

#[derive(Debug, Serialize)]
pub struct ElementRefMap<'a> {
    pub type_: HashMap<String, Vec<&'a Element>>,
    pub nature: HashMap<String, Vec<&'a Element>>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipRefMap<'a> {
    pub type_: HashMap<String, Vec<&'a Relationship>>,
    pub nature: HashMap<String, Vec<&'a Relationship>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementCount {
    pub element: String,
    pub count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementCounts {
    pub value: Vec<ElementCount>,
}

impl ModelDictionary {
    pub fn from(model: &ModelData, model_versions: Vec<ModelVersionNumber>) -> Self {
        let start_time = Instant::now();

        /* Generate stats */
        let element_type_count =
            generate_element_count_by(&model.elements, |obj| obj.get_type()).unwrap_or_default();
        let element_nature_count =
            generate_element_count_by(&model.elements, |obj| obj.get_nature()).unwrap_or_default();
        let rel_type_count = generate_element_count_by(&model.relationships, |obj| obj.get_type())
            .unwrap_or_default();
        let rel_nature_count =
            generate_element_count_by(&model.relationships, |obj| obj.get_nature())
                .unwrap_or_default();

        //Log time
        let elapsed_time = start_time.elapsed();
        println!(
            "[Execution time] {} - {:?}",
            "ModelDictionary::from", elapsed_time
        );

        // Construct output
        ModelDictionary {
            model_id: model.model_id.clone(),
            version: model.version,
            model_stats: ModelStats {
                elements_stats: Some(CubsObjectReport {
                    all_count: model.elements.len() as u32,
                    by_type: element_type_count,
                    by_nature: element_nature_count,
                }),

                relationships_stats: Some(CubsObjectReport {
                    all_count: model.relationships.len() as u32,
                    by_type: rel_type_count,
                    by_nature: rel_nature_count,
                }),
                version: model.version,
            },
            model_versions: model_versions,
        }
    }

    pub fn get_element_types(&self) -> Vec<String> {
        match &self.model_stats.elements_stats {
            Some(r) => r.by_type.value.iter().map(|c| c.element.clone()).collect(),
            None => vec![],
        }
    }

    pub fn get_element_nature(&self) -> Vec<String> {
        match &self.model_stats.elements_stats {
            Some(r) => r
                .by_nature
                .value
                .iter()
                .map(|c| c.element.clone())
                .collect(),
            None => vec![],
        }
    }
}

impl ModelStats {
    /* Get stats from elements ref */
    pub fn from_elements(elements: &Vec<&Element>) -> Option<Self> {
        let mut by_type: HashMap<String, u32> = HashMap::new();
        let mut by_nature: HashMap<String, u32> = HashMap::new();
        let count = elements.len() as u32;

        for element in elements {
            *by_type.entry(element.type_.clone()).or_insert(0 as u32) += 1;
            *by_nature.entry(element.nature.clone()).or_insert(0 as u32) += 1;
        }

        // Construct Output
        let mut element_count_by_type: Vec<ElementCount> = by_type
            .into_iter()
            .map(|(element, count)| ElementCount { element, count })
            .collect();
        element_count_by_type.sort_by(|a, b| b.count.cmp(&a.count));

        let mut element_count_by_nature: Vec<ElementCount> = by_nature
            .into_iter()
            .map(|(element, count)| ElementCount { element, count })
            .collect();
        element_count_by_nature.sort_by(|a, b| b.count.cmp(&a.count));

        Some(Self {
            elements_stats: Some(CubsObjectReport {
                all_count: count,
                by_type: ElementCounts {
                    value: element_count_by_type,
                },
                by_nature: ElementCounts {
                    value: element_count_by_nature,
                },
            }),
            relationships_stats: None,
            version: 0,
        })
    }
}

// Helper method
pub fn generate_array_field_count(value: &Value, field_name: &str) -> Option<ElementCounts> {
    let array = value.as_array()?;

    let mut type_counts: HashMap<String, u32> = HashMap::new();

    for element in array {
        if let Some(type_value) = element.get(field_name) {
            if let Some(type_str) = type_value.as_str() {
                *type_counts.entry(type_str.to_owned()).or_insert(0) += 1;
            }
        }
    }

    if type_counts.is_empty() {
        return None;
    }

    let mut counts: Vec<ElementCount> = type_counts
        .into_iter()
        .map(|(element, count)| ElementCount { element, count })
        .collect();
    counts.sort_by(|a, b| b.count.cmp(&a.count));

    Some(ElementCounts { value: counts })
}

fn _get_json_array_len(value: &Value) -> u32 {
    if let Some(array) = value.as_array() {
        array.len() as u32
    } else {
        0
    }
}

pub fn generate_element_count_by<T, F>(
    cubs_objects: &Vec<T>,
    key_getter: F,
) -> Option<ElementCounts>
where
    T: CusObject,
    F: Fn(&T) -> String,
{
    // Partition into count map
    let partition_map = cubs_objects.iter().fold(HashMap::new(), |mut acc, obj| {
        let key = key_getter(obj);
        let value = acc.entry(key).or_insert_with(|| 0u32);
        *value += 1;
        acc
    });

    if partition_map.is_empty() {
        return None;
    }

    //Order with descending order
    let mut element_counts: Vec<ElementCount> = partition_map
        .into_iter()
        .map(|(element, count)| ElementCount { element, count })
        .collect();
    element_counts.sort_by(|a, b| b.count.cmp(&a.count));

    Some(ElementCounts {
        value: element_counts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_array_with_types() {
        let json = json!([
            {"type": "cube", "id": 1},
            {"type": "sphere", "id": 2},
            {"type": "cube", "id": 3},
            {"type": "cube", "id": 4}
        ]);

        let result = generate_array_field_count(&json, "type").unwrap();

        assert_eq!(result.value.len(), 2);
        assert_eq!(result.value[0].element, "cube");
        assert_eq!(result.value[0].count, 3);
        assert_eq!(result.value[1].element, "sphere");
        assert_eq!(result.value[1].count, 1);
    }

    #[test]
    fn test_empty_array() {
        let json = json!([]);
        let result = generate_array_field_count(&json, "type");
        assert!(result.is_none());
    }

    #[test]
    fn test_non_array_input() {
        let json = json!({"not": "array"});
        let result = generate_array_field_count(&json, "type");
        assert!(result.is_none());
    }

    #[test]
    fn test_array_without_type_fields() {
        let json = json!([{"id": 1}, {"name": "test"}]);
        let result = generate_array_field_count(&json, "type");
        assert!(result.is_none());
    }
}
