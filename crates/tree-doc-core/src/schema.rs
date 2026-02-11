use crate::error::{Diagnostic, Location, Rule, Severity};

static TIER0_SCHEMA_STR: &str = include_str!("../../../schemas/tier0.schema.json");
static TIER1_SCHEMA_STR: &str = include_str!("../../../schemas/tier1.schema.json");

use std::sync::OnceLock;

fn tier0_schema() -> &'static jsonschema::Validator {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: serde_json::Value = serde_json::from_str(TIER0_SCHEMA_STR)
            .expect("embedded tier0 schema is valid JSON");
        jsonschema::validator_for(&schema).expect("embedded tier0 schema is valid")
    })
}

fn tier1_schema() -> &'static jsonschema::Validator {
    static VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: serde_json::Value = serde_json::from_str(TIER1_SCHEMA_STR)
            .expect("embedded tier1 schema is valid JSON");
        jsonschema::validator_for(&schema).expect("embedded tier1 schema is valid")
    })
}

pub fn validate_schema(value: &serde_json::Value) -> Vec<Diagnostic> {
    let validator = tier0_schema();
    let mut diagnostics = Vec::new();

    for error in validator.iter_errors(value) {
        diagnostics.push(Diagnostic {
            rule: Rule::SchemaValidation,
            message: format!("{error}"),
            location: Location::Root,
            severity: Severity::Error,
        });
    }

    diagnostics
}

pub fn detect_tier(value: &serde_json::Value) -> u8 {
    if value.get("trees").is_some() {
        return 2;
    }
    let has_tier1_fields = value.get("minReaderVersion").is_some()
        || value.get("features").is_some()
        || value.get("metadata").is_some();
    if has_tier1_fields {
        let _ = tier1_schema();
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_minimal_passes_schema() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let diags = validate_schema(&value);
        assert!(diags.is_empty(), "expected no errors, got: {diags:?}");
    }

    #[test]
    fn valid_story_passes_schema() {
        let json = include_str!("../../../examples/story.tree.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let diags = validate_schema(&value);
        assert!(diags.is_empty(), "expected no errors, got: {diags:?}");
    }

    #[test]
    fn missing_fields_fails_schema() {
        let json = include_str!("../../../examples/invalid/missing-fields.tree.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let diags = validate_schema(&value);
        assert!(!diags.is_empty(), "expected schema errors for missing fields");
        assert!(diags.iter().all(|d| d.severity == Severity::Error));
    }

    #[test]
    fn detect_tier0() {
        let json = include_str!("../../../examples/minimal.tree.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(detect_tier(&value), 0);
    }

    #[test]
    fn detect_tier1() {
        let json = include_str!("../../../examples/story.tree.json");
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(detect_tier(&value), 1);
    }

    #[test]
    fn extra_fields_pass_schema() {
        let json = r#"{
            "formatVersion": "1.0",
            "rootNodeId": "n1",
            "nodes": [{"id": "n1", "content": "hello"}],
            "edges": [],
            "customField": "should be allowed"
        }"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let diags = validate_schema(&value);
        assert!(diags.is_empty());
    }

    #[test]
    fn wrong_type_fails_schema() {
        let json = r#"{
            "formatVersion": 1,
            "rootNodeId": "n1",
            "nodes": [],
            "edges": []
        }"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let diags = validate_schema(&value);
        assert!(!diags.is_empty());
    }
}
