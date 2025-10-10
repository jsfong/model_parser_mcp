use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::time::Instant;

use crate::model::{
    app_state::QuickCache, config::PageConfig, cubs_model::{self, Element, FacetType, ModelData, ModelVersionNumber}, element_graph::ElementGraph, element_graph_parser::ElementGraphParser, element_parser::ElementConnectorBuilder, model_dict::ModelDictionary, model_error::ModelError, parser, utils::Utils

};

pub struct ModelParser<'a> {
    model_cache: QuickCache<ModelData>,
    graph_cache: QuickCache<ElementGraph>,
    pg_pool: &'a Pool<Postgres>,
}

#[derive(Default)]
pub struct ModelQueryResult {
    pub data: String,
    pub duration: String,
    pub page_count: Page,
    pub total_result_count: usize,
}

#[derive(Default)]
pub struct Page {
    pub elements_per_page: usize,
    pub total_page: usize,
    pub current_page: usize,
}

impl<'a> ModelParser<'a> {
    pub fn new(
        model_cache: QuickCache<ModelData>,
        graph_cache: QuickCache<ElementGraph>,
        pg_pool: &'a Pool<Postgres>,
    ) -> Self {
        ModelParser {
            model_cache,
            graph_cache,
            pg_pool,
        }
    }

    //Get model Stats
    pub async fn get_model_stats(
        &self,
        model_id: &str,
        version_number: &str,
    ) -> Result<ModelDictionary, ModelError> {
        let model_id = model_id.to_owned();
        println!(
            "[ModelParser - get_model_stats] Getting model stats of {} with version {}",
            model_id,
            version_number
        );
        let start_time = Instant::now();

        //Read all model version
        let model_version = parser::read_model_data_versions(self.pg_pool, &model_id)
            .await
            .unwrap_or_default();
        Utils::log_time(start_time, "Read model data version");

        // Get model
        let model_data = self
            .get_model(&model_id, version_number, &model_version)
            .await
            .ok_or_else(|| ModelError::UnableToReadModel)?;
        Utils::log_time(start_time, "Read model data");

        //Build relationship graph
        self.build_relationship_graph(&model_id, model_data.version, &model_data);

        //Build stats
        let dict = ModelDictionary::from(&model_data, model_version);
        Utils::log_time(start_time, "Get model stats");
        println!(
            "[parse_model] Successfully parse model with id {} \n",
            model_id
        );

        Ok(dict)
    }

    //Query model
    pub async fn query_model(
        &self,
        model_id: String,
        version_number: String,
        id: String,
        is_parse_subgraph: bool,
        types: String,
        natures: String,
        query: String,
        depth: usize,
        page_config: PageConfig,
        facet_type: String,
        is_detail: bool,
    ) -> Result<ModelQueryResult, ModelError> {
        println!(
            "[ModelParser - query_model] Querying model: {} with query: {} with depth: {} and page config: {:?}",
            model_id,
            query,
            depth,
            page_config,
        );

        // Input Validation
        if model_id.is_empty() {
            return Err(ModelError::InvalidInput(
                "Model id is empty nothing to query.".to_string(),
            ));
        }

        if is_parse_subgraph && id.is_empty() {
            return Err(ModelError::InvalidInput(
                "Unable to parse subgraph. Please provide element id in Id".to_string(),
            ));
        }

        let start_time = Instant::now();

        // Get model
        let model_version = parser::read_model_data_versions(self.pg_pool, &model_id)
            .await
            .unwrap_or_default();
        let model_data = self
            .get_model(&model_id, &version_number, &model_version)
            .await
            .ok_or_else(|| ModelError::UnableToReadModel)?;
        Utils::log_time(start_time, "Read model data");

        // Perform Filtering
        // Get subgraph
        let subgraph_elements: Vec<String> = if is_parse_subgraph && !id.is_empty() {
            let graph_cache = &self.graph_cache;
            let graph = if let Some(graph) = graph_cache.get(&model_id, &version_number) {
                graph
            } else {
                // Build graph if not found
                let built_graph = ElementConnectorBuilder::build_graph(
                    &model_data.elements,
                    &model_data.relationships,
                )?;

                // Add to cache
                graph_cache.insert(&model_id, &version_number, &built_graph);

                built_graph
            };

            ElementGraphParser::parse_graph(&graph, &id, 0, 99)
                .map(|g| g.get_all_elements())
                .unwrap_or(Vec::new())
        } else {
            Vec::new()
        };

        //Filter id
        println!(
            "[ModelParser - query_model] Pre-filter element count {}",
            subgraph_elements.len()
        );
        let mut filtered_elements = if id.is_empty() {
            model_data.get_elements()
        } else if is_parse_subgraph && !subgraph_elements.is_empty() {
            //Parsing subgraph
            println!("[ModelParser - query_model] - Filtering sub graph");
            model_data.get_element_with_filter(|e: &Element| subgraph_elements.contains(&e.id))
        } else {
            //Filter id only
            println!("[ModelParser - query_model] - Filtering id");
            model_data
                .get_element_with_id(&id)
                .map(|e| vec![e])
                .unwrap_or_else(|| Vec::new())
        };

        //filter nature
        filtered_elements.retain(|e| match natures.as_str() {
            "All" => true,
            _ => *e.nature == natures,
        });

        //filter type
        filtered_elements.retain(|e| match types.as_str() {
            "All" => true,
            _ => *e.type_ == types,
        });

        //Apply json pointer
        let facet_type: Option<FacetType> = match facet_type.as_str() {
            "dynamicFacets" => Some(FacetType::DynamicFacets),
            "coreFacets" => Some(FacetType::CoreFacets),
            "facets" => Some(FacetType::Facets),
            _ => None,
        };
        println!(
            "[ModelParser - query_model] Applying json pointer facet type: {:?} pointer: {} with detail: {}",
            facet_type, &query, is_detail
        );
        let filtered_elements = if facet_type.is_some() {
            ModelData::get_json_values(filtered_elements, facet_type, &query, is_detail)
        } else {
            filtered_elements
                .iter()
                .map(|e| serde_json::to_value(e).unwrap_or_default())
                .filter(|v| *v != Value::Null)
                .collect()
        };
        let filtered_element_len = filtered_elements.len();

        //Limit & Pagination
        let elements_chunks: Vec<&[Value]> = filtered_elements
            .chunks(page_config.elements_per_page)
            .collect();
        let page = Page {
            elements_per_page: page_config.elements_per_page,
            total_page: elements_chunks.len(),
            current_page: page_config.page_to_get,
        };
        let limited_query_result = elements_chunks[page_config.page_to_get - 1];
        println!(
            "[ModelParser - query_model] Getting page {} of {} with total element {}",
            page.current_page,
            page.total_page,
            filtered_element_len
        );

        //Depth
        println!(
            "[ModelParser - query_model] truncating {} elements to depth {}",
            limited_query_result.len(),
            depth
        );
        let elements = match depth > 0 {
            true => {
                let filtered_element = cubs_model::truncate_value(&limited_query_result, depth);
                serde_json::to_string_pretty(&filtered_element).unwrap()
            }
            false => serde_json::to_string_pretty(&limited_query_result).unwrap(),
        };

        // Log time
        Utils::log_time(start_time, "ModelParser - query_model");
        let elapsed_time = start_time.elapsed();

        //Construct output
        Ok(ModelQueryResult {
            data: elements,
            duration: format!(
                "Query model took {} ms",
                elapsed_time.as_millis().to_string()
            ),
            page_count: page,
            total_result_count: filtered_elements.len(),
        })
    }

    async fn get_model(
        &self,
        model_id: &String,
        version_number: &str,
        model_versions: &Vec<ModelVersionNumber>,
    ) -> Option<ModelData> {
        //Read saved model
        let version_number = ModelParser::get_version_number(version_number, model_versions);
        let model_data =
            parser::read_model_data(self.pg_pool, &self.model_cache, &model_id, version_number)
                .await
                .ok();

        model_data
    }

    fn build_relationship_graph(
        &self,
        model_id: &String,
        version_number: u32,
        model_data: &ModelData,
    ) -> Option<ElementGraph> {
        println!("[ModelParser - build_relationship_graph] Building relationship graph");
        let start_time = Instant::now();
        let graph_cache = &self.graph_cache;
        let elements = &model_data.elements;
        let relationships = &model_data.relationships;

        //Check if exist in cache
        let existing_graph = graph_cache.get(&model_id, &version_number.to_string());

        //If not exist, building graph
        if version_number != 0 && existing_graph.is_none() {
            println!("[ModelParser - build_relationship_graph] Graph not found cache. Building...");
            let graph = match ElementConnectorBuilder::build_graph(elements, relationships) {
                Ok(graph) => {
                    graph_cache.insert(&model_id, &version_number.to_string(), &graph);
                    Some(graph)
                }
                Err(_) => None,
            };
            Utils::log_time(start_time, "Building Relationship graph");
            graph
        } else {
            existing_graph
        }
    }

    fn get_version_number(version_number: &str, model_versions: &Vec<ModelVersionNumber>) -> i32 {
        version_number
            .parse::<i32>()
            .unwrap_or_else(|_| model_versions.first().map_or(0, |v| v.vers_no))
    }
}
