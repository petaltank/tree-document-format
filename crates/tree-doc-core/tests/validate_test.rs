use tree_doc_core::error::Rule;
use tree_doc_core::validate_document;

#[test]
fn valid_minimal() {
    let json = include_str!("../../../examples/minimal.tree.json");
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
    assert_eq!(result.stats.node_count, 3);
    assert_eq!(result.stats.edge_count, 2);
    assert_eq!(result.stats.tier, 0);
}

#[test]
fn valid_story() {
    let json = include_str!("../../../examples/story.tree.json");
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
    assert_eq!(result.stats.node_count, 7);
    assert_eq!(result.stats.tier, 1);
    assert_eq!(result.stats.trunk_length, 4);
}

#[test]
fn valid_empty_document() {
    let json = include_str!("../../../examples/empty-document.tree.json");
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
    assert_eq!(result.stats.node_count, 1);
    assert_eq!(result.stats.edge_count, 0);
    assert_eq!(result.stats.trunk_length, 0);
}

#[test]
fn invalid_duplicate_ids() {
    let json = include_str!("../../../examples/invalid/duplicate-ids.tree.json");
    let result = validate_document(json).unwrap();
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|d| d.rule == Rule::DuplicateNodeId));
}

#[test]
fn invalid_dangling_edge() {
    let json = include_str!("../../../examples/invalid/dangling-edge.tree.json");
    let result = validate_document(json).unwrap();
    assert!(!result.is_valid);
    let dangling_count = result
        .errors
        .iter()
        .filter(|d| d.rule == Rule::DanglingEdge)
        .count();
    // n3 (source) and n99 (target) are both nonexistent
    assert_eq!(dangling_count, 2);
}

#[test]
fn invalid_trunk_cycle() {
    let json = include_str!("../../../examples/invalid/trunk-cycle.tree.json");
    let result = validate_document(json).unwrap();
    assert!(!result.is_valid);
    assert!(result.errors.iter().any(|d| d.rule == Rule::TrunkCycle));
}

#[test]
fn invalid_missing_fields() {
    let json = include_str!("../../../examples/invalid/missing-fields.tree.json");
    let result = validate_document(json).unwrap();
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|d| d.rule == Rule::SchemaValidation));
}

#[test]
fn invalid_orphan_nodes() {
    let json = include_str!("../../../examples/invalid/orphan-node.tree.json");
    let result = validate_document(json).unwrap();
    // Orphans are advisories, not errors
    assert!(result.is_valid);
    let orphans: Vec<_> = result
        .advisories
        .iter()
        .filter(|d| d.rule == Rule::OrphanNode)
        .collect();
    assert_eq!(orphans.len(), 2);
}

#[test]
fn invalid_general_cycle() {
    let json = include_str!("../../../examples/invalid/general-cycle.tree.json");
    let result = validate_document(json).unwrap();
    // General cycles are warnings, not errors
    assert!(result.is_valid);
    assert!(result
        .warnings
        .iter()
        .any(|d| d.rule == Rule::GeneralCycle));
}

#[test]
fn malformed_json_returns_error() {
    let result = validate_document("{not valid json}");
    assert!(result.is_err());
}

#[test]
fn wrong_shape_array_not_object() {
    let json = r#"[{"id": "n1"}]"#;
    let result = validate_document(json).unwrap();
    assert!(!result.is_valid);
}

#[test]
fn extra_fields_accepted() {
    let json = r#"{
        "formatVersion": "1.0",
        "rootNodeId": "n1",
        "nodes": [{"id": "n1", "content": "hi", "customField": 42}],
        "edges": [],
        "topLevelCustom": true
    }"#;
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
}

#[test]
fn multiple_trunk_edges_from_one_node() {
    // Only the last trunk edge wins in our HashMap, but this should still validate
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
            {"source": "n1", "target": "n3", "isTrunk": true}
        ]
    }"#;
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
}

#[test]
fn linear_document_no_branches() {
    let json = r#"{
        "formatVersion": "1.0",
        "rootNodeId": "n1",
        "nodes": [
            {"id": "n1", "content": "Chapter 1"},
            {"id": "n2", "content": "Chapter 2"},
            {"id": "n3", "content": "Chapter 3"}
        ],
        "edges": [
            {"source": "n1", "target": "n2", "isTrunk": true},
            {"source": "n2", "target": "n3", "isTrunk": true}
        ]
    }"#;
    let result = validate_document(json).unwrap();
    assert!(result.is_valid);
    assert_eq!(result.stats.trunk_length, 2);
    assert_eq!(result.stats.branch_count, 0);
}
