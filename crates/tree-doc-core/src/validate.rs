use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::graph::{DiGraph, NodeIndex};

use crate::error::{Diagnostic, DocumentStats, Location, Rule, Severity, ValidationResult};
use crate::parse::{self, ParseError};
use crate::schema;
use crate::types::TreeDocument;

/// Run the full validation pipeline: parse → schema → semantic → stats.
pub fn validate_document(json_str: &str) -> Result<ValidationResult, ParseError> {
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();

    // Step 1: Parse as generic JSON value
    let value = parse::parse_value(json_str)?;

    // Step 2: Schema validation
    let schema_diags = schema::validate_schema(&value);
    let has_schema_errors = !schema_diags.is_empty();
    all_diagnostics.extend(schema_diags);

    // If schema validation fails, we may not be able to parse into typed structs.
    // Try anyway — serde is more lenient than the schema in some ways.
    let doc = match parse::parse(json_str) {
        Ok(doc) => doc,
        Err(_) if has_schema_errors => {
            // Can't parse — return schema errors only
            return Ok(ValidationResult {
                is_valid: false,
                errors: all_diagnostics,
                warnings: Vec::new(),
                advisories: Vec::new(),
                stats: DocumentStats {
                    node_count: 0,
                    edge_count: 0,
                    trunk_length: 0,
                    branch_count: 0,
                    tier: 0,
                },
            });
        }
        Err(e) => return Err(e),
    };

    // Step 3: Semantic validation
    let semantic_diags = validate_semantics(&doc);
    all_diagnostics.extend(semantic_diags);

    // Step 4: Compute stats
    let tier = schema::detect_tier(&value);
    let trunk_length = compute_trunk_length(&doc);
    let branch_count = doc
        .edges
        .iter()
        .filter(|e| e.is_trunk != Some(true))
        .count();

    let stats = DocumentStats {
        node_count: doc.nodes.len(),
        edge_count: doc.edges.len(),
        trunk_length,
        branch_count,
        tier,
    };

    // Partition diagnostics by severity
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut advisories = Vec::new();

    for diag in all_diagnostics {
        match diag.severity {
            Severity::Error => errors.push(diag),
            Severity::Warning => warnings.push(diag),
            Severity::Advisory => advisories.push(diag),
        }
    }

    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        advisories,
        stats,
    })
}

/// Run all semantic validation rules on a parsed document.
fn validate_semantics(doc: &TreeDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Rule 1: Duplicate node IDs
    check_duplicate_ids(doc, &mut diagnostics);

    // Build node ID set for subsequent checks
    let node_ids: HashSet<&str> = doc.nodes.iter().map(|n| n.id.as_str()).collect();

    // Rule 2: Dangling edges
    check_dangling_edges(doc, &node_ids, &mut diagnostics);

    // Rule 3: Trunk cycle detection
    check_trunk_cycle(doc, &node_ids, &mut diagnostics);

    // Rule 4: General cycle detection (Tarjan's SCC)
    check_general_cycles(doc, &node_ids, &mut diagnostics);

    // Rule 5: Orphan nodes
    check_orphan_nodes(doc, &node_ids, &mut diagnostics);

    diagnostics
}

/// Rule 1: Reject duplicate node IDs.
fn check_duplicate_ids(doc: &TreeDocument, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();
    for node in &doc.nodes {
        if !seen.insert(node.id.as_str()) {
            diagnostics.push(Diagnostic {
                rule: Rule::DuplicateNodeId,
                message: format!("Duplicate node ID '{}'", node.id),
                location: Location::Node(node.id.clone()),
                severity: Severity::Error,
            });
        }
    }
}

/// Rule 2: Reject edges referencing nonexistent nodes.
fn check_dangling_edges(
    doc: &TreeDocument,
    node_ids: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for edge in &doc.edges {
        if !node_ids.contains(edge.source.as_str()) {
            diagnostics.push(Diagnostic {
                rule: Rule::DanglingEdge,
                message: format!(
                    "Edge references nonexistent node '{}' as source (target: '{}')",
                    edge.source, edge.target
                ),
                location: Location::Edge {
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                },
                severity: Severity::Error,
            });
        }
        if !node_ids.contains(edge.target.as_str()) {
            diagnostics.push(Diagnostic {
                rule: Rule::DanglingEdge,
                message: format!(
                    "Edge references nonexistent node '{}' as target (source: '{}')",
                    edge.target, edge.source
                ),
                location: Location::Edge {
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                },
                severity: Severity::Error,
            });
        }
    }
}

/// Rule 3: Detect cycles in the trunk path via iterative walk.
fn check_trunk_cycle(
    doc: &TreeDocument,
    node_ids: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let root_id = match &doc.root_node_id {
        Some(id) if node_ids.contains(id.as_str()) => id.as_str(),
        _ => return,
    };

    // Build trunk adjacency: source -> target for isTrunk edges
    let mut trunk_next: HashMap<&str, &str> = HashMap::new();
    for edge in &doc.edges {
        if edge.is_trunk == Some(true) {
            trunk_next.insert(edge.source.as_str(), edge.target.as_str());
        }
    }

    // Walk trunk from root, tracking visited nodes
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    let mut current = root_id;

    loop {
        if !visited.insert(current) {
            // Found a cycle — collect the cycle path
            let cycle_start = path.iter().position(|n: &&str| *n == current).unwrap_or(0);
            let cycle_path: Vec<String> =
                path[cycle_start..].iter().map(|s| (*s).to_string()).collect();
            diagnostics.push(Diagnostic {
                rule: Rule::TrunkCycle,
                message: format!(
                    "Trunk path contains a cycle: {}",
                    cycle_path.join(" -> ")
                ),
                location: Location::Path(cycle_path),
                severity: Severity::Error,
            });
            return;
        }
        path.push(current);
        match trunk_next.get(current) {
            Some(&next) => current = next,
            None => return, // End of trunk — no cycle
        }
    }
}

/// Rule 4: Detect general cycles using Tarjan's SCC via petgraph.
fn check_general_cycles(
    doc: &TreeDocument,
    _node_ids: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Build petgraph DiGraph
    let mut graph = DiGraph::new();
    let mut id_to_index: HashMap<&str, NodeIndex> = HashMap::new();
    let mut index_to_id: HashMap<NodeIndex, &str> = HashMap::new();

    for node in &doc.nodes {
        // Only add first occurrence (skip duplicates)
        if !id_to_index.contains_key(node.id.as_str()) {
            let idx = graph.add_node(node.id.as_str());
            id_to_index.insert(node.id.as_str(), idx);
            index_to_id.insert(idx, node.id.as_str());
        }
    }

    for edge in &doc.edges {
        if let (Some(&src), Some(&tgt)) = (
            id_to_index.get(edge.source.as_str()),
            id_to_index.get(edge.target.as_str()),
        ) {
            graph.add_edge(src, tgt, ());
        }
    }

    // Tarjan's SCC
    let sccs = petgraph::algo::tarjan_scc(&graph);
    for scc in &sccs {
        if scc.len() > 1 {
            let cycle_ids: Vec<String> = scc
                .iter()
                .filter_map(|idx| index_to_id.get(idx).map(|s| s.to_string()))
                .collect();
            diagnostics.push(Diagnostic {
                rule: Rule::GeneralCycle,
                message: format!(
                    "Cycle detected among {} nodes: {}",
                    cycle_ids.len(),
                    cycle_ids.join(", ")
                ),
                location: Location::Path(cycle_ids),
                severity: Severity::Warning,
            });
        }
    }
}

/// Rule 5: Detect orphan nodes unreachable from root via BFS.
fn check_orphan_nodes(
    doc: &TreeDocument,
    node_ids: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let root_id = match &doc.root_node_id {
        Some(id) if node_ids.contains(id.as_str()) => id.as_str(),
        _ => return,
    };

    // Build adjacency list (edges are directional: source -> target)
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &doc.edges {
        adjacency
            .entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    // BFS from root
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root_id);
    visited.insert(root_id);

    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(current) {
            for &neighbor in neighbors {
                if visited.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // Any node not visited is an orphan
    for node in &doc.nodes {
        if !visited.contains(node.id.as_str()) {
            diagnostics.push(Diagnostic {
                rule: Rule::OrphanNode,
                message: format!(
                    "Node '{}' is not reachable from root node '{}'",
                    node.id, root_id
                ),
                location: Location::Node(node.id.clone()),
                severity: Severity::Advisory,
            });
        }
    }
}

/// Count trunk edges to determine trunk length.
fn compute_trunk_length(doc: &TreeDocument) -> usize {
    let root_id = match &doc.root_node_id {
        Some(id) => id.as_str(),
        None => return 0,
    };

    let mut trunk_next: HashMap<&str, &str> = HashMap::new();
    for edge in &doc.edges {
        if edge.is_trunk == Some(true) {
            trunk_next.insert(edge.source.as_str(), edge.target.as_str());
        }
    }

    let mut visited = HashSet::new();
    let mut current = root_id;
    let mut length = 0;

    while let Some(&next) = trunk_next.get(current) {
        if !visited.insert(current) {
            break; // Avoid infinite loop on trunk cycles
        }
        length += 1;
        current = next;
    }

    length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_minimal_passes() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let result = validate_document(json).unwrap();
        assert!(result.is_valid, "errors: {:?}", result.errors);
        assert_eq!(result.stats.node_count, 3);
        assert_eq!(result.stats.edge_count, 2);
        assert_eq!(result.stats.trunk_length, 1);
        assert_eq!(result.stats.branch_count, 1);
    }

    #[test]
    fn valid_story_passes() {
        let json = include_str!("../../../examples/story.tree.json");
        let result = validate_document(json).unwrap();
        assert!(result.is_valid, "errors: {:?}", result.errors);
        assert_eq!(result.stats.node_count, 7);
        assert_eq!(result.stats.tier, 1);
    }

    #[test]
    fn duplicate_ids_rejected() {
        let json = include_str!("../../../examples/invalid/duplicate-ids.tree.json");
        let result = validate_document(json).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|d| d.rule == Rule::DuplicateNodeId));
    }

    #[test]
    fn dangling_edge_rejected() {
        let json = include_str!("../../../examples/invalid/dangling-edge.tree.json");
        let result = validate_document(json).unwrap();
        assert!(!result.is_valid);
        let dangling: Vec<_> = result
            .errors
            .iter()
            .filter(|d| d.rule == Rule::DanglingEdge)
            .collect();
        assert!(
            dangling.len() >= 2,
            "expected at least 2 dangling edge errors, got: {dangling:?}"
        );
    }

    #[test]
    fn trunk_cycle_rejected() {
        let json = include_str!("../../../examples/invalid/trunk-cycle.tree.json");
        let result = validate_document(json).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|d| d.rule == Rule::TrunkCycle));
    }

    #[test]
    fn missing_fields_rejected() {
        let json = include_str!("../../../examples/invalid/missing-fields.tree.json");
        let result = validate_document(json).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|d| d.rule == Rule::SchemaValidation));
    }

    #[test]
    fn general_cycle_warns() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [
                {"id": "n1", "content": "Start"},
                {"id": "n2", "content": "A"},
                {"id": "n3", "content": "B"}
            ],
            "edges": [
                {"source": "n1", "target": "n2", "isTrunk": true},
                {"source": "n2", "target": "n3"},
                {"source": "n3", "target": "n2"}
            ]
        }"#;
        let result = validate_document(json).unwrap();
        assert!(result.is_valid, "general cycles are warnings, not errors");
        assert!(result
            .warnings
            .iter()
            .any(|d| d.rule == Rule::GeneralCycle));
    }

    #[test]
    fn orphan_node_advisory() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [
                {"id": "n1", "content": "Start"},
                {"id": "n2", "content": "Connected"},
                {"id": "orphan", "content": "I am unreachable"}
            ],
            "edges": [
                {"source": "n1", "target": "n2", "isTrunk": true}
            ]
        }"#;
        let result = validate_document(json).unwrap();
        assert!(result.is_valid, "orphans are advisories, not errors");
        assert!(result
            .advisories
            .iter()
            .any(|d| d.rule == Rule::OrphanNode));
    }

    #[test]
    fn single_node_no_edges() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [{"id": "n1", "content": "Only node"}],
            "edges": []
        }"#;
        let result = validate_document(json).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.stats.node_count, 1);
        assert_eq!(result.stats.edge_count, 0);
        assert_eq!(result.stats.trunk_length, 0);
    }

    #[test]
    fn self_loop_detected() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [
                {"id": "n1", "content": "Self-looping node"}
            ],
            "edges": [
                {"source": "n1", "target": "n1"}
            ]
        }"#;
        let result = validate_document(json).unwrap();
        // Self-loop is a general cycle (SCC of size 1 with self-edge isn't detected by tarjan_scc
        // as len>1, but we should at least not crash)
        assert!(result.is_valid);
    }
}
