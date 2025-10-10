use std::collections::HashMap;

// Graph hold all the connection

#[derive(Clone, Debug)]
pub struct ElementGraph {
    // Each element contain one connector
    connectors: HashMap<String, ElementConnector>,
    connected_relationship: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ElementConnector {
    element_id: String,
    in_ids: Vec<Path>,
    out_ids: Vec<Path>,
}

#[derive(Clone, Debug)]
pub struct Path(pub String, pub String); //Relationship id : id

//Implementation
impl ElementGraph {
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
            connected_relationship: Vec::new(),
        }
    }

    // Add connector without connection
    pub fn add_connector(&mut self, id: &str) {
        if !self.connectors.contains_key(id) {
            self.connectors.insert(
                id.to_owned(),
                ElementConnector {
                    element_id: id.to_owned(),
                    in_ids: Vec::new(),
                    out_ids: Vec::new(),
                },
            );
        }
    }

    pub fn push_connector(&mut self, id: &str, connector: ElementConnector) {
        self.connectors.insert(id.to_owned(), connector);
    }

    // Connect element
    pub fn connect(&mut self, relationship_id: &str, from_id: &str, to_id: &str) {
        let mut connected_in = false;
        let mut connected_out = false;

        // From Obj --> add output
        if let Some(from_obj) = self.connectors.get_mut(from_id) {
            from_obj
                .out_ids
                .push(Path(relationship_id.to_owned(), to_id.to_owned()));
            connected_in = true;
        }

        // To Obj --> add input
        if let Some(to_obj) = self.connectors.get_mut(to_id) {
            to_obj
                .in_ids
                .push(Path(relationship_id.to_owned(), from_id.to_owned()));
            connected_out = true;
        }

        if connected_in && connected_out {
            self.connected_relationship.push(relationship_id.to_owned());
        }
    }

    pub fn get_connection(&self, id: &str) -> Option<&ElementConnector> {
        self.connectors.get(id)
    }

    pub fn get_connection_count(&self) -> usize {
        self.connectors.len()
    }

    pub fn get_connected_relationship_count(&self) -> usize {
        self.connected_relationship.len()
    }

    pub fn add_connected_relationship(&mut self, relationship_id: &str) {
        self.connected_relationship.push(relationship_id.to_owned());
    }

    pub fn get_connected_relationship(&self) -> Vec<String> {
        self.connected_relationship.clone()
    }

    pub fn get_all_elements(&self) -> Vec<String> {
        self.connectors.keys().map(|k| k.clone()).collect()
    }
}

impl ElementConnector {
    pub fn get_element_id(&self) -> &str {
        &self.element_id
    }

    pub fn get_in_id(&self) -> Vec<&Path> {
        self.in_ids.iter().map(|id| id).collect()
    }

    pub fn get_out_id(&self) -> Vec<&Path> {
        self.out_ids.iter().map(|id| id).collect()
    }

    pub fn clear_in_id(&mut self) {
        self.in_ids.clear();
    }

    pub fn clear_out_id(&mut self) {
        self.out_ids.clear();
    }

    pub fn retain_in_id(&mut self, id: &str) {
        self.in_ids.retain(|p| p.1 == id);
    }

    pub fn retain_out_id(&mut self, id: &str) {
        self.out_ids.retain(|p| p.1 == id);
    }

    pub fn is_in_ids_empty(&self) -> bool {
        self.in_ids.is_empty()
    }

    pub fn is_out_ids_empty(&self) -> bool {
        self.out_ids.is_empty()
    }
}

// Trait
impl<'a> std::fmt::Display for ElementConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = &self.element_id;

        //Parent
        for path in &self.in_ids {
            let id = &path.1;
            let rel_id = &path.0;
            writeln!(f, "<{}> -- ({}) --> ", id, rel_id)?;
        }

        //Current
        writeln!(f, "              [{}] ", id)?;

        // Child
        for path in &self.out_ids {
            let id = &path.1;
            let rel_id = &path.0;
            writeln!(f, "                 -- ({}) --> <{}>", rel_id, id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::element_graph::{ElementConnector, ElementGraph};

    #[test]
    fn test_new() {
        // Build
        let mut graph = ElementGraph::new();
        graph.add_connector("c1");
        graph.add_connector("c2");
        graph.add_connector("c3");
        graph.add_connector("c4");
        graph.add_connector("c5");
        graph.connect("r1", "c1", "c3");
        graph.connect("r2", "c2", "c3");
        graph.connect("r3", "c3", "c4");
        graph.connect("r4", "c3", "c5");

        // Print
        let c1 = graph.get_connection("c1");
        let c2 = graph.get_connection("c2");
        let c3 = graph.get_connection("c3");
        let c4 = graph.get_connection("c4");
        let c5 = graph.get_connection("c5");

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

        assert!(true);
    }

    #[test]
    fn test_parse_graph() {
        // Build
        let mut graph = ElementGraph::new();
        graph.add_connector("c1");
        graph.add_connector("c2");
        graph.add_connector("c3");
        graph.add_connector("c4");
        graph.add_connector("c5");
        graph.add_connector("c6");
        graph.connect("r1", "c1", "c2");
        graph.connect("r2", "c2", "c3");
        graph.connect("r3", "c2", "c4");
        graph.connect("r4", "c4", "c5");
        graph.connect("r5", "c4", "c6");

        // Print
        let c1 = graph.get_connection("c1");

        if let Some(c) = c1 {
            println!("--- Print C1 ----");
            println!("{}", c);
        }

        assert!(true);
    }
}
