use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeDocument {
    pub format_version: String,
    pub root_node_id: Option<String>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    // Tier 1
    pub min_reader_version: Option<String>,
    pub features: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    // Tier 2
    pub trees: Option<HashMap<String, TreeDescriptor>>,
    pub embedding_ref: Option<EmbeddingRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub status: Option<String>,
    pub tree_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub is_trunk: Option<bool>,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub edge_type: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub tree_id: Option<String>,
    pub link_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeDescriptor {
    pub root_node_id: String,
    pub label: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingRef {
    pub format: String,
    pub path: Option<String>,
}
