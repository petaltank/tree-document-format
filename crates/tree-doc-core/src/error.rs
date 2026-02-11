use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Advisory,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Advisory => write!(f, "advisory"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule {
    SchemaValidation,
    DuplicateNodeId,
    DanglingEdge,
    TrunkCycle,
    GeneralCycle,
    OrphanNode,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::SchemaValidation => write!(f, "schema-validation"),
            Rule::DuplicateNodeId => write!(f, "duplicate-node-id"),
            Rule::DanglingEdge => write!(f, "dangling-edge"),
            Rule::TrunkCycle => write!(f, "trunk-cycle"),
            Rule::GeneralCycle => write!(f, "general-cycle"),
            Rule::OrphanNode => write!(f, "orphan-node"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Location {
    Root,
    Node(String),
    Edge { source: String, target: String },
    Path(Vec<String>),
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Root => write!(f, "(document root)"),
            Location::Node(id) => write!(f, "node '{id}'"),
            Location::Edge { source, target } => write!(f, "edge '{source}' -> '{target}'"),
            Location::Path(ids) => write!(f, "path: {}", ids.join(" -> ")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule: Rule,
    pub message: String,
    pub location: Location,
    pub severity: Severity,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {} (at {})",
            self.severity, self.rule, self.message, self.location
        )
    }
}

#[derive(Debug, Clone)]
pub struct DocumentStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub trunk_length: usize,
    pub branch_count: usize,
    pub tier: u8,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub advisories: Vec<Diagnostic>,
    pub stats: DocumentStats,
}
