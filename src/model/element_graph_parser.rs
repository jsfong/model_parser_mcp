use std::collections::HashMap;
use crate::model::{
        config::{OutputGraph, OutputLine, OutputToken, RelationshipDirection}, cubs_model::ModelData, element_graph::{ElementConnector, ElementGraph, Path}, model_error::ModelError
    };

pub struct ElementGraphParser;

impl ElementGraphParser {
    pub fn parse_graph(
        full_graph: &ElementGraph,
        target_element_id: &str,
        ancestor_level_limit: u32,
        children_level_limit: u32,
    ) -> Result<ElementGraph, ModelError> {
        let mut partial_graph = ElementGraph::new();

        // Find target element connector
        println!("[ElementGraphParser - parse_graph ] retrieving {} from {} connection", target_element_id, full_graph.get_connection_count());
        let target_connector =
            full_graph
                .get_connection(target_element_id)
                .ok_or(ModelError::ParsingError(format!(
                    "Elemennt: {}",
                    target_element_id
                )))?;

        // Iterate up from target to parent
        Self::parse_parent(
            full_graph,
            &mut partial_graph,
            target_connector,
            1,
            ancestor_level_limit,
        );

        // Iterate down from target to child
        Self::parse_child(
            full_graph,
            &mut partial_graph,
            target_connector,
            1,
            children_level_limit,
        );

        // Add target
        let mut target_connector_cloned = target_connector.clone();
        if ancestor_level_limit == 0 {
            target_connector_cloned.clear_in_id();
        }
        if children_level_limit == 0 {
            target_connector_cloned.clear_out_id();
        }
        partial_graph.push_connector(&target_connector.get_element_id(), target_connector_cloned);

        Ok(partial_graph)
    }

    fn parse_parent(
        source_graph: &ElementGraph,
        target_graph: &mut ElementGraph,
        current_element_connnector: &ElementConnector,
        current_level: u32,
        limit: u32,
    ) {
        // Stopping condition
        if current_level > limit {
            return;
        }

        let current_element_id = current_element_connnector.get_element_id();
        let ids: Vec<&Path> = current_element_connnector.get_in_id();
        if ids.is_empty() {
            return;
        }

        // Executing  For every id in the path add to the target graph
        for path in ids {
            let id = &path.1;
            let relationship_id = &path.0;
            if let Some(parent_connector) = source_graph.get_connection(&id) {
                println!("[ElementGraphParser - parse_parent] Adding element {} with relationship {} to graph at level: {}", parent_connector.get_element_id(), relationship_id, current_level);

                let mut cloned_parent_connector = parent_connector.clone();
                // Truncate the path if reach limit
                if current_level == limit {
                    cloned_parent_connector.clear_in_id();
                }

                // prune out other branch
                cloned_parent_connector.retain_out_id(current_element_id);

                // Add to target graph
                target_graph.push_connector(&id.to_owned(), cloned_parent_connector);
                target_graph.add_connected_relationship(&relationship_id.to_owned());

                // Recrusive call
                ElementGraphParser::parse_parent(
                    source_graph,
                    target_graph,
                    parent_connector,
                    current_level + 1,
                    limit,
                    // &path_retrieval,
                );
            } else {
                println!("[ElementGraphParser - parse_parent] Error parsing {}", id);
                break;
            }
        }
    }

    fn parse_child(
        source_graph: &ElementGraph,
        target_graph: &mut ElementGraph,
        current_element_connnector: &ElementConnector,
        current_level: u32,
        limit: u32,
    ) {
        // Stopping condition
        if current_level > limit {
            return;
        }

        let ids: Vec<&Path> = current_element_connnector.get_out_id();
        if ids.is_empty() {
            return;
        }

        // Executing  For every id in the path add to the target graph
        for path in ids {
            let id = &path.1;
            let relationship_id = &path.0;
            if let Some(parent_connector) = source_graph.get_connection(&id) {
                println!("[ElementGraphParser - parse_child] Adding element {} with relationship {} to graph at level: {}", parent_connector.get_element_id(), relationship_id, current_level);

                let mut cloned_parent_connector = parent_connector.clone();
                // Truncate the path if reach limit
                if current_level == limit {
                    cloned_parent_connector.clear_out_id();
                }

                // Add to target graph
                target_graph.push_connector(&id.to_owned(), cloned_parent_connector);
                target_graph.add_connected_relationship(&relationship_id.to_owned());

                // Recrusive call
                ElementGraphParser::parse_child(
                    source_graph,
                    target_graph,
                    parent_connector,
                    current_level + 1,
                    limit,
                );
            } else {
                println!("[ElementGraphParser - parse_child] Error parsing {}", id);
                break;
            }
        }
    }

    pub fn build_output(
        full_graph: &ElementGraph,
        target_element_id: &str,
        model_data: &ModelData,
    ) -> Result<OutputGraph<String>, ModelError> {
        // Peform DFS
        let mut output = OutputGraph {
            parent_lines: Vec::new(),
            child_lines: Vec::new(),
            elements_data: HashMap::new(),
        };
        let current_element = full_graph.get_connection(target_element_id);

        // Collect traversed element
        let mut traversed_element_ids: Vec<String> = Vec::new();

        // Child
        Self::dfs(
            full_graph,
            current_element,
            &mut traversed_element_ids,
            0,
            5,
            &RelationshipDirection::Child,
            &mut output,
        );

        // Parent
        Self::dfs(
            full_graph,
            current_element,
            &mut traversed_element_ids,
            0,
            2,
            &RelationshipDirection::Parent,
            &mut output,
        );
        output.parent_lines.reverse();

        // Retrieve elements
        let element_map = traversed_element_ids
            .iter()
            .filter_map(|id| {
                model_data
                    .get_element_with_id(id)
                    .and_then(|element| serde_json::to_value(element.get_common_fields_values_map()).ok())
                    .map(|value| (id.clone(), value))
            })
            .collect();
        output.elements_data = element_map;

        Ok(output)
    }

    fn dfs(
        full_graph: &ElementGraph,
        current_element: Option<&ElementConnector>,
        traversed_element_ids: &mut Vec<String>,
        level: u32,
        limit: u32,
        direcion: &RelationshipDirection,
        result: &mut OutputGraph<String>,
    ) {
        if let Some(current_element) = current_element {
            //Stop when over limit
            if level > limit {
                return;
            }

            // Marked as traversed
            traversed_element_ids.push(current_element.get_element_id().to_owned());

            // Generate token
            let mut output_line = OutputLine::new();

            // Tab
            for _ in 0..level {
                output_line.push(OutputToken::Tab);
            }

            // Arrow
            if level != 0 {
                match direcion {
                    RelationshipDirection::Parent => output_line.push(OutputToken::InArrow),
                    RelationshipDirection::Child => output_line.push(OutputToken::OutArrow),
                };
            }

            //Value
            output_line.push(OutputToken::Value(
                current_element.get_element_id().to_owned(),
            ));

            //Output
            match direcion {
                RelationshipDirection::Parent => result.parent_lines.push(output_line),
                RelationshipDirection::Child => result.child_lines.push(output_line),
            };

            //Child
            let childs = match direcion {
                RelationshipDirection::Parent => current_element.get_in_id(),
                RelationshipDirection::Child => current_element.get_out_id(),
            };

            for child in childs {
                let element_id = &child.1;
                let _rel_id = &child.0;
                let next_element = full_graph.get_connection(&element_id);
                Self::dfs(
                    full_graph,
                    next_element,
                    traversed_element_ids,
                    level + 1,
                    limit,
                    direcion,
                    result,
                );
            }
            //Stop when no child to tranverse
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::model::{
        cubs_model::ModelData,
        element_graph::{ElementConnector, ElementGraph},
        element_graph_parser::ElementGraphParser,
    };

    #[test]
    fn test_new() {
        // Build
        let mut graph = ElementGraph::new();
        graph.add_connector("c1");
        graph.add_connector("c2");
        graph.add_connector("c3");
        graph.add_connector("c4");
        graph.add_connector("c5");
        graph.add_connector("c6");
        graph.add_connector("c7");
        graph.add_connector("c8");
        graph.add_connector("c9");
        graph.connect("r1", "c1", "c3");
        graph.connect("r2", "c2", "c4");
        graph.connect("r3", "c3", "c5");
        graph.connect("r4", "c4", "c5");
        graph.connect("r5", "c5", "c6");
        graph.connect("r6", "c5", "c7");
        graph.connect("r7", "c7", "c8");
        graph.connect("r8", "c7", "c9");

        // Parse with 1 level
        let target = "c5";
        let parse_graph = ElementGraphParser::parse_graph(&graph, target, 0, 1).unwrap();

        // Print
        let c1 = parse_graph.get_connection("c1");
        let c2 = parse_graph.get_connection("c2");
        let c3 = parse_graph.get_connection("c3");
        let c4 = parse_graph.get_connection("c4");
        let c5 = parse_graph.get_connection("c5");
        let c6 = parse_graph.get_connection("c6");
        let c7 = parse_graph.get_connection("c7");
        let c8 = parse_graph.get_connection("c8");
        let c9 = parse_graph.get_connection("c9");

        if let Some(c) = c1 {
            println!("--- Print C1 ----");
            println!("{}", c);
        }
        println!();

        if let Some(c) = c2 {
            println!("--- Print C2 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c3 {
            println!("--- Print C3 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c4 {
            println!("--- Print C4 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c5 {
            println!("--- Print C5 ----");
            println!("{}", c);
        }
        println!();

        if let Some(c) = c6 {
            println!("--- Print C6 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c7 {
            println!("--- Print C7 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c8 {
            println!("--- Print C8 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c9 {
            println!("--- Print C9 ----");
            println!("{}", c);
        }

        assert!(true);

        println!("Rel {:?}", parse_graph.get_connected_relationship());
    }

    #[test]
    fn test_build_output() {
        // Build
        let mut graph = ElementGraph::new();
        graph.add_connector("c1");
        graph.add_connector("c2");
        graph.add_connector("c3");
        graph.add_connector("c4");
        graph.add_connector("c5");
        graph.add_connector("c6");
        graph.add_connector("c7");
        graph.add_connector("c8");
        graph.add_connector("c9");
        graph.connect("r1", "c1", "c3");
        graph.connect("r2", "c2", "c4");
        graph.connect("r3", "c3", "c5");
        graph.connect("r4", "c4", "c5");
        graph.connect("r5", "c5", "c6");
        graph.connect("r6", "c5", "c7");
        graph.connect("r7", "c7", "c8");
        graph.connect("r8", "c7", "c9");

        // Parse with 1 level
        let target = "c5";
        let parse_graph = ElementGraphParser::parse_graph(&graph, target, 0, 2).unwrap();

        // Print
        let c1 = parse_graph.get_connection("c1");
        let c2 = parse_graph.get_connection("c2");
        let c3 = parse_graph.get_connection("c3");
        let c4 = parse_graph.get_connection("c4");
        let c5 = parse_graph.get_connection("c5");
        let c6 = parse_graph.get_connection("c6");
        let c7 = parse_graph.get_connection("c7");
        let c8 = parse_graph.get_connection("c8");
        let c9 = parse_graph.get_connection("c9");

        if let Some(c) = c1 {
            println!("--- Print C1 ----");
            println!("{}", c);
        }
        println!();

        if let Some(c) = c2 {
            println!("--- Print C2 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c3 {
            println!("--- Print C3 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c4 {
            println!("--- Print C4 ----");
            println!("{}", c);
        }

        println!();

        if let Some(c) = c5 {
            println!("--- Print C5 ----");
            println!("{}", c);
        }
        println!();

        if let Some(c) = c6 {
            println!("--- Print C6 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c7 {
            println!("--- Print C7 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c8 {
            println!("--- Print C8 ----");
            println!("{}", c);
        }

        assert!(true);

        if let Some(c) = c9 {
            println!("--- Print C9 ----");
            println!("{}", c);
        }

        let output =
            ElementGraphParser::build_output(&parse_graph, target, &ModelData::default())
                .unwrap();

        println!("OUTPUT {:?}", output);

        assert!(true);
    }
}
