use std::time::Instant;

use crate::model::{
    cubs_model::{Element, Relationship},
    element_graph::ElementGraph,
    model_error::ModelError,
};

pub struct ElementConnectorBuilder;

impl ElementConnectorBuilder {
    pub fn build_graph(
        elements: &[Element],
        relationship: &[Relationship],
    ) -> Result<ElementGraph, ModelError> {
        if elements.is_empty() || relationship.is_empty() {
            return Err(ModelError::ModelGraphBuildingError(
                "Unable to build graph due to empty element or empty relationship".to_string(),
            ));
        }

        let start_time = Instant::now();
        let mut graph = ElementGraph::new();

        // For each elements build a connector
        println!("[ElementConnectorBuilder - build_graph: Building graph]");
        elements.iter().for_each(|e| {
            graph.add_connector(&e.id);
        });
        println!(
            "[ElementConnectorBuilder - build_graph: Built {} connector]",
            graph.get_connection_count()
        );

        // For each relationship connect connector
        relationship.iter().for_each(|r| {
            graph.connect(&r.id, &r.source_id, &r.target_id);
        });
        println!(
            "[ElementConnectorBuilder - build_graph: Built {} relationship]",
            graph.get_connected_relationship_count()
        );

        // Validation
        // graph connector == number element
        let elements_size = elements.len();
        let graph_elements_size = graph.get_connection_count();
        if elements_size != graph_elements_size {
            return Err(ModelError::ModelGraphBuildingError(format!(
                "Error building graph due to invalid element number. Provided: {} vs built:{} ",
                elements_size, graph_elements_size
            )));
        }

        // graph connected relationship == relationship
        let relationship_size = relationship.len();
        let graph_relationship_size = graph.get_connected_relationship_count();
        if relationship_size != graph_relationship_size {
            return Err(ModelError::ModelGraphBuildingError(format!(
                "Error building graph due to invalid relationship number. Provided: {} vs built:{} ",
                relationship_size, graph_relationship_size
            )));
        }

        //Log time
        let elapsed_time = start_time.elapsed();
        println!(
            "[Execution time] {} - {:?}",
            "ElementConnectorBuilder - build_graph", elapsed_time
        );

        Ok(graph)
    }
}
