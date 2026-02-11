use tree_doc_core::{build_trunk_view, parse};

#[test]
fn minimal_trunk_view() {
    let json = include_str!("../../../examples/minimal.tree.json");
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    assert_eq!(view.steps.len(), 2);
    assert_eq!(view.steps[0].node_id, "n1");
    assert_eq!(view.steps[0].branch_count, 1);
    assert!(!view.steps[0].is_terminal);
    assert_eq!(view.steps[1].node_id, "n2");
    assert!(view.steps[1].is_terminal);
}

#[test]
fn story_trunk_view() {
    let json = include_str!("../../../examples/story.tree.json");
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    assert_eq!(view.title, "The Enchanted Garden");
    assert_eq!(view.steps.len(), 5);
    assert_eq!(view.steps[0].node_id, "start");
    assert_eq!(view.steps[4].node_id, "ending");
    assert!(view.steps[4].is_terminal);
}

#[test]
fn empty_document_view() {
    let json = include_str!("../../../examples/empty-document.tree.json");
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    assert_eq!(view.steps.len(), 1);
    assert!(view.steps[0].is_terminal);
    assert_eq!(view.steps[0].content, "");
}

#[test]
fn linear_document_view() {
    let json = r#"{
        "formatVersion": "1.0",
        "rootNodeId": "n1",
        "nodes": [
            {"id": "n1", "content": "A"},
            {"id": "n2", "content": "B"},
            {"id": "n3", "content": "C"}
        ],
        "edges": [
            {"source": "n1", "target": "n2", "isTrunk": true},
            {"source": "n2", "target": "n3", "isTrunk": true}
        ]
    }"#;
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    assert_eq!(view.steps.len(), 3);
    for step in &view.steps[..2] {
        assert_eq!(step.branch_count, 0);
        assert!(!step.is_terminal);
    }
    assert!(view.steps[2].is_terminal);
}

#[test]
fn no_trunk_markers_single_step() {
    let json = r#"{
        "formatVersion": "1.0",
        "rootNodeId": "n1",
        "nodes": [
            {"id": "n1", "content": "Root"},
            {"id": "n2", "content": "Child"}
        ],
        "edges": [
            {"source": "n1", "target": "n2"}
        ]
    }"#;
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    // Without isTrunk markers, trunk walk stops at root
    assert_eq!(view.steps.len(), 1);
    assert!(view.steps[0].is_terminal);
    assert_eq!(view.steps[0].branch_count, 1);
}

#[test]
fn trunk_view_with_branch_labels() {
    let json = r#"{
        "formatVersion": "1.0",
        "rootNodeId": "n1",
        "nodes": [
            {"id": "n1", "content": "Choose your path"},
            {"id": "n2", "content": "Main path"},
            {"id": "n3", "content": "Side quest A"},
            {"id": "n4", "content": "Side quest B"}
        ],
        "edges": [
            {"source": "n1", "target": "n2", "isTrunk": true},
            {"source": "n1", "target": "n3", "label": "Go left"},
            {"source": "n1", "target": "n4", "label": "Go right"}
        ]
    }"#;
    let doc = parse(json).unwrap();
    let view = build_trunk_view(&doc).unwrap();

    assert_eq!(view.steps[0].branch_count, 2);
    assert!(view.steps[0].branch_labels.contains(&"Go left".to_string()));
    assert!(view.steps[0].branch_labels.contains(&"Go right".to_string()));
}
