use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::{char, fmt};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelResponse {
    pub data: ModelData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelData {
    pub schema_version: String,
    pub model_id: String,
    pub site_model_id: String,
    pub version: u32,
    #[serde(alias = "cubsObjects", deserialize_with = "null_to_empty_vec")]
    #[serde(alias = "cubsObjects")]
    pub elements: Vec<Element>,
    // pub elements: Value,
    #[serde(deserialize_with = "null_to_empty_vec")]
    pub relationships: Vec<Relationship>,
    // pub relationships: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelVersionNumber {
    pub vers_no: i32, //postgres int4 is map back to i32
}

pub trait CusObject {
    fn get_nature(&self) -> String;
    fn get_type(&self) -> String;
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_dynamic_facet(&self) -> &HashMap<String, serde_json::Value>;
    fn get_facet(&self) -> &HashMap<String, serde_json::Value>;
    fn get_core_facet(&self) -> &HashMap<String, serde_json::Value>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub id: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub nature: String,
    #[serde(default)]
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub dynamic_facets: HashMap<String, serde_json::Value>,
    pub facets: HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub core_facets: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub nature: String,
    #[serde(default)]
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub dynamic_facets: HashMap<String, serde_json::Value>,
    pub facets: HashMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub core_facets: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub enum FacetType {
    CoreFacets,
    DynamicFacets,
    Facets,
}

impl fmt::Display for FacetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FacetType::CoreFacets => write!(f, "core_facets"),
            FacetType::DynamicFacets => write!(f, "dynamic_facets"),
            FacetType::Facets => write!(f, "mk3_facets"),
        }
    }
}

impl CusObject for Element {
    fn get_nature(&self) -> String {
        self.nature.clone()
    }

    fn get_type(&self) -> String {
        self.type_.clone()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_dynamic_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.dynamic_facets
    }

    fn get_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.facets
    }

    fn get_core_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.core_facets
    }
}

impl Element {
    pub fn get_json_value(
        &self,
        facet_type: &FacetType,
        pointer: &str,
        is_show_element_id: bool,
    ) -> Option<Value> {
        // Construct filtered element
        let filtered_element = match facet_type {
            FacetType::CoreFacets => Element {
                dynamic_facets: HashMap::new(),
                facets: HashMap::new(),
                ..self.clone()
            },
            FacetType::DynamicFacets => Element {
                core_facets: HashMap::new(),
                facets: HashMap::new(),
                ..self.clone()
            },
            FacetType::Facets => Element {
                core_facets: HashMap::new(),
                dynamic_facets: HashMap::new(),
                ..self.clone()
            },
        };

        let mut combine_core_facet = filtered_element.core_facets.clone();
        let facets_map: &HashMap<String, Value> = match facet_type {
            FacetType::CoreFacets => {
                // Add common field into core facet for parsing
                let common_fields: HashMap<String, Value> = self.get_common_fields_values_map();
                combine_core_facet.extend(common_fields);
                &combine_core_facet
            }
            FacetType::DynamicFacets => &filtered_element.dynamic_facets,
            FacetType::Facets => &filtered_element.facets,
        };

        // Return of no query need to be perform
        if pointer.is_empty() || facets_map.is_empty() {
            return match facets_map.is_empty() {
                true => None,
                false => serde_json::to_value(&filtered_element).ok(),
            };
        }

        //Perform json pointer
        let mut facets_map_value = serde_json::to_value(facets_map).unwrap();
        let ptr = facets_map_value.pointer_mut(pointer);
        match ptr {
            Some(v) => {
                let result = if is_show_element_id {
                    let e = FilteredElementResult::from(&filtered_element, v.take());
                    serde_json::to_value(e).ok()
                } else {
                    Some(v.take())
                };
                result
            }
            None => None,
        }
    }

    pub fn get_common_fields_values_map(&self) -> HashMap<String, serde_json::Value> {
        let mut fields_values_map: HashMap<String, serde_json::Value> = HashMap::new();

        Self::set_value_to_map(& mut fields_values_map, "id", &self.id.clone());
        Self::set_value_to_map(& mut fields_values_map, "type", &self.type_.clone());
        Self::set_value_to_map(& mut fields_values_map, "nature", &self.nature.clone());
        Self::set_value_to_map(& mut fields_values_map, "name", &self.name.clone());

        fields_values_map
    }

    fn set_value_to_map(map: &mut HashMap<String, serde_json::Value>, key: &str, fields: &str) {
        let field_value = serde_json::to_value(fields).ok();
        if let Some(value) = field_value {
            map.insert(key.to_string(), value);
        }
    }
}

impl CusObject for Relationship {
    fn get_nature(&self) -> String {
        self.nature.clone()
    }

    fn get_type(&self) -> String {
        self.type_.clone()
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_dynamic_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.dynamic_facets
    }

    fn get_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.facets
    }

    fn get_core_facet(&self) -> &HashMap<String, serde_json::Value> {
        &self.core_facets
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilteredElementResult {
    pub id: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub nature: String,
    #[serde(default)]
    pub name: String,
    pub version: u32,
    pub filtered_result: Value,
}

impl FilteredElementResult {
    pub fn from(element: &Element, result: Value) -> FilteredElementResult {
        FilteredElementResult {
            id: element.id.clone(),
            type_: element.type_.clone(),
            nature: element.nature.clone(),
            name: element.name.clone(),
            version: element.version,
            filtered_result: result,
        }
    }
}

//Deserializer
fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let opt = Option::<Vec<T>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

//TODO refactor using trait
impl ModelData {
    pub fn get_elements(&self) -> Vec<&Element> {
        self.elements.iter().collect()
    }

    pub fn get_element_with_id(&self, id: &str) -> Option<&Element> {
        let id_filter = |e: &Element| e.id == id;
        let r: Vec<&Element> = self.get_element_with_filter(id_filter);
        r.first().copied()
    }

    pub fn get_element_with_filter<F>(&self, filter: F) -> Vec<&Element>
    where
        F: Fn(&Element) -> bool,
    {
        self.elements.iter().filter(|e| filter(e)).collect()
    }

    pub fn get_json_values(
        elements: Vec<&Element>,
        facet_type: Option<FacetType>,
        pointer: &str,
        is_show_element_id: bool,
    ) -> Vec<Value> {
        println!(
            "[Element - get_json_values] for {:?} with path {}",
            facet_type, pointer
        );

        if let Some(facet_type) = facet_type {
            elements
                .iter()
                .filter_map(|e| e.get_json_value(&facet_type, pointer, is_show_element_id))
                .collect()
        } else {
            vec![]
        }
    }
}

pub fn truncate_value(values: &[Value], truncate_depth: usize) -> Vec<Value> {
    let result = values
        .iter()
        .map(|v| truncate(v, truncate_depth, 0))
        .collect();

    result
}

fn truncate(value: &Value, max_depth: usize, current_depth: usize) -> Value {
    if current_depth >= max_depth {
        return match value {
            Value::Array(_) => Value::Null,
            Value::Object(_) => Value::Null,
            other => other.clone(),
        };
    }

    match value {
        Value::Array(arr) => {
            let new_array: Vec<Value> = arr
                .iter()
                .map(|v| truncate(v, max_depth, current_depth + 1))
                .collect();
            Value::Array(new_array)
        }

        Value::Object(map) => {
            let mut new_map = Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), truncate(v, max_depth, current_depth + 1));
            }
            Value::Object(new_map)
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_truncate() {
        // Test with a simple JSON object
        let json = r#"{"a": 1, "b": {"c": 2}}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let result = truncate(&value, 2, 0);
        let result_string = result.to_string();
        assert_eq!(result_string, r#"{"a":1,"b":{"c":2}}"#);
    }

    #[test]
    fn test_truncate2() {
        // Test with a simple JSON object
        let json = r#"{"a": 1,"b": {"c": {"d": 2}}}"#;
        let value: Value = serde_json::from_str(json).unwrap();
        let result = truncate(&value, 2, 0);
        let result_string = result.to_string();
        assert_eq!(result_string, r#"{"a":1,"b":{"c":null}}"#);
    }
}
