use serde::Serialize;
use wasm_bindgen::prelude::*;

fn to_js<T: Serialize>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn validate(json_str: &str) -> JsValue {
    let result = match tree_doc_core::validate_document(json_str) {
        Ok(r) => r,
        Err(e) => {
            return to_js(&serde_json::json!({
                "error": format!("{e}"),
                "isValid": false,
            }));
        }
    };

    to_js(&serde_json::json!({
        "isValid": result.is_valid,
        "errors": result.errors.iter().map(|d| serde_json::json!({
            "rule": d.rule.to_string(),
            "message": d.message,
            "location": d.location.to_string(),
            "severity": d.severity.to_string(),
        })).collect::<Vec<_>>(),
        "warnings": result.warnings.iter().map(|d| serde_json::json!({
            "rule": d.rule.to_string(),
            "message": d.message,
            "location": d.location.to_string(),
            "severity": d.severity.to_string(),
        })).collect::<Vec<_>>(),
        "advisories": result.advisories.iter().map(|d| serde_json::json!({
            "rule": d.rule.to_string(),
            "message": d.message,
            "location": d.location.to_string(),
            "severity": d.severity.to_string(),
        })).collect::<Vec<_>>(),
        "stats": serde_json::json!({
            "nodeCount": result.stats.node_count,
            "edgeCount": result.stats.edge_count,
            "trunkLength": result.stats.trunk_length,
            "branchCount": result.stats.branch_count,
            "tier": result.stats.tier,
        }),
    }))
}

#[wasm_bindgen]
pub fn view(json_str: &str) -> JsValue {
    let doc = match tree_doc_core::parse(json_str) {
        Ok(d) => d,
        Err(e) => {
            return to_js(&serde_json::json!({ "error": format!("{e}") }));
        }
    };

    let trunk_view = match tree_doc_core::build_trunk_view(&doc) {
        Ok(v) => v,
        Err(e) => {
            return to_js(&serde_json::json!({ "error": e }));
        }
    };

    to_js(&serde_json::json!({
        "title": trunk_view.title,
        "stats": trunk_view.stats,
        "steps": trunk_view.steps.iter().map(|s| serde_json::json!({
            "nodeId": s.node_id,
            "content": s.content,
            "branchCount": s.branch_count,
            "branchLabels": s.branch_labels,
            "isTerminal": s.is_terminal,
            "trunkTarget": s.trunk_target,
        })).collect::<Vec<_>>(),
    }))
}

#[wasm_bindgen]
pub fn info(json_str: &str) -> JsValue {
    let result = match tree_doc_core::validate_document(json_str) {
        Ok(r) => r,
        Err(e) => {
            return to_js(&serde_json::json!({ "error": format!("{e}") }));
        }
    };

    to_js(&serde_json::json!({
        "nodeCount": result.stats.node_count,
        "edgeCount": result.stats.edge_count,
        "trunkLength": result.stats.trunk_length,
        "branchCount": result.stats.branch_count,
        "tier": result.stats.tier,
        "isValid": result.is_valid,
    }))
}
