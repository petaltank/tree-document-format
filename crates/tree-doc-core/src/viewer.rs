use std::collections::HashMap;

use crate::types::TreeDocument;

#[derive(Debug)]
pub struct TrunkView {
    pub title: String,
    pub stats: String,
    pub steps: Vec<TrunkStep>,
}

#[derive(Debug)]
pub struct TrunkStep {
    pub node_id: String,
    pub content: String,
    pub branch_count: usize,
    pub branch_labels: Vec<String>,
    pub is_terminal: bool,
    pub trunk_target: Option<String>,
}

pub fn build_trunk_view(doc: &TreeDocument) -> Result<TrunkView, String> {
    let root_id = doc
        .root_node_id
        .as_deref()
        .ok_or_else(|| "Document has no rootNodeId".to_string())?;

    // Build lookup maps
    let node_map: HashMap<&str, &crate::types::Node> =
        doc.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Build trunk adjacency: source -> target for isTrunk edges
    let mut trunk_next: HashMap<&str, &str> = HashMap::new();
    for edge in &doc.edges {
        if edge.is_trunk == Some(true) {
            trunk_next.insert(edge.source.as_str(), edge.target.as_str());
        }
    }

    // Build branch info: source -> list of (target, label) for non-trunk edges
    let mut branches: HashMap<&str, Vec<(&str, Option<&str>)>> = HashMap::new();
    for edge in &doc.edges {
        if edge.is_trunk != Some(true) {
            branches
                .entry(edge.source.as_str())
                .or_default()
                .push((edge.target.as_str(), edge.label.as_deref()));
        }
    }

    // Walk trunk from root
    let mut steps = Vec::new();
    let mut current = root_id;
    let mut visited = std::collections::HashSet::new();

    loop {
        if !visited.insert(current) {
            break; // Avoid infinite loop on trunk cycles
        }

        let node = node_map
            .get(current)
            .ok_or_else(|| format!("Root node '{}' not found in nodes array", current))?;

        let node_branches = branches.get(current).cloned().unwrap_or_default();
        let branch_labels: Vec<String> = node_branches
            .iter()
            .filter_map(|(_, label)| label.map(|l| l.to_string()))
            .collect();

        let next = trunk_next.get(current).copied();
        let is_terminal = next.is_none();

        steps.push(TrunkStep {
            node_id: current.to_string(),
            content: node.content.clone(),
            branch_count: node_branches.len(),
            branch_labels,
            is_terminal,
            trunk_target: next.map(|s| s.to_string()),
        });

        match next {
            Some(n) => current = n,
            None => break,
        }
    }

    let title = doc
        .metadata
        .as_ref()
        .and_then(|m| m.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("Untitled Document")
        .to_string();

    let stats = format!(
        "{} nodes, {} edges",
        doc.nodes.len(),
        doc.edges.len()
    );

    Ok(TrunkView {
        title,
        stats,
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn minimal_trunk_view() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let doc = parse::parse(json).unwrap();
        let view = build_trunk_view(&doc).unwrap();

        assert_eq!(view.title, "Untitled Document");
        assert_eq!(view.steps.len(), 2); // n1 -> n2 (trunk), n3 is branch
        assert_eq!(view.steps[0].node_id, "n1");
        assert_eq!(view.steps[0].branch_count, 1); // n3 is a branch
        assert!(!view.steps[0].is_terminal);
        assert_eq!(view.steps[0].trunk_target, Some("n2".to_string()));
        assert_eq!(view.steps[1].node_id, "n2");
        assert!(view.steps[1].is_terminal);
        assert_eq!(view.steps[1].branch_count, 0);
    }

    #[test]
    fn story_trunk_view() {
        let json = include_str!("../../../examples/story.tree.json");
        let doc = parse::parse(json).unwrap();
        let view = build_trunk_view(&doc).unwrap();

        assert_eq!(view.title, "The Enchanted Garden");
        // Trunk: start -> enter -> fountain -> wish -> ending
        assert_eq!(view.steps.len(), 5);
        assert_eq!(view.steps[0].node_id, "start");
        assert_eq!(view.steps[0].branch_count, 1); // "climb" branch
        assert_eq!(view.steps[0].branch_labels, vec!["Climb the wall"]);
        assert_eq!(view.steps[4].node_id, "ending");
        assert!(view.steps[4].is_terminal);
    }

    #[test]
    fn single_node_view() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [{"id": "n1", "content": "Only node"}],
            "edges": []
        }"#;
        let doc = parse::parse(json).unwrap();
        let view = build_trunk_view(&doc).unwrap();

        assert_eq!(view.steps.len(), 1);
        assert!(view.steps[0].is_terminal);
        assert_eq!(view.steps[0].branch_count, 0);
    }
}
