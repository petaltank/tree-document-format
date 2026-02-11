use crate::types::TreeDocument;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

pub fn parse(json_str: &str) -> Result<TreeDocument, ParseError> {
    let doc: TreeDocument = serde_json::from_str(json_str)?;
    Ok(doc)
}

pub fn parse_value(json_str: &str) -> Result<serde_json::Value, ParseError> {
    let value: serde_json::Value = serde_json::from_str(json_str)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_document() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let doc = parse(json).unwrap();
        assert_eq!(doc.format_version, "1.0");
        assert_eq!(doc.root_node_id.as_deref(), Some("n1"));
        assert_eq!(doc.nodes.len(), 3);
        assert_eq!(doc.edges.len(), 2);
    }

    #[test]
    fn parse_tier1_document() {
        let json = include_str!("../../../examples/story.tree.json");
        let doc = parse(json).unwrap();
        assert_eq!(doc.min_reader_version.as_deref(), Some("1.0"));
        assert_eq!(doc.features.as_ref().unwrap(), &["labels"]);
        assert!(doc.metadata.is_some());
    }

    #[test]
    fn roundtrip_serde() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let doc = parse(json).unwrap();
        let serialized = serde_json::to_string_pretty(&doc).unwrap();
        let doc2 = parse(&serialized).unwrap();
        assert_eq!(doc.nodes.len(), doc2.nodes.len());
        assert_eq!(doc.edges.len(), doc2.edges.len());
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn parse_value_returns_value() {
        let json = r#"{"formatVersion": "1.0"}"#;
        let value = parse_value(json).unwrap();
        assert_eq!(value["formatVersion"], "1.0");
    }

    #[test]
    fn edge_type_renames_correctly() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let doc = parse(json).unwrap();
        let trunk_edge = &doc.edges[0];
        assert_eq!(trunk_edge.is_trunk, Some(true));
        // Serialize back and verify "type" field name
        let val = serde_json::to_value(&doc.edges[0]).unwrap();
        assert!(val.get("isTrunk").is_some());
    }
}
