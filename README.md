# Tree Document Format

> **This project is a work in progress and is not ready for use.** APIs, file format details, and CLI behavior are all subject to change without notice. Do not depend on this in any project yet.

A portable JSON-based file format for representing **branching documents** — trees of text nodes connected by edges. Think interactive fiction, game dialogue, decision trees, planning systems, or any tool working with non-linear text.

This repository contains the **reference implementation**: a validator, a viewer, published JSON Schemas, and a browser-based drag-and-drop viewer powered by WASM.

**Status:** Early development. See the [roadmap](#roadmap) below for what's done and what's still planned.

## What is a `.tree.json` file?

A `.tree.json` file describes a directed graph of text nodes. Each node has an `id` and `content`. Edges connect nodes, and edges marked `isTrunk` define the primary reading path through the document.

Here's a minimal example (`examples/minimal.tree.json`):

```json
{
  "formatVersion": "1.0",
  "rootNodeId": "n1",
  "nodes": [
    { "id": "n1", "content": "You stand at a crossroads in the forest." },
    { "id": "n2", "content": "You take the left path. A quiet village appears ahead." },
    { "id": "n3", "content": "You take the right path. The trees grow darker." }
  ],
  "edges": [
    { "source": "n1", "target": "n2", "isTrunk": true },
    { "source": "n1", "target": "n3" }
  ]
}
```

The trunk path goes `n1 -> n2`. Node `n3` is a branch — an alternative path the reader could take.

## Prerequisites

- **Rust 1.83+** (install via [rustup](https://rustup.rs/))
- For WASM builds: `wasm32-unknown-unknown` target and `wasm-bindgen-cli`

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

## Quick Start

Clone the repo and build:

```bash
git clone <repo-url> tree-document-format
cd tree-document-format
cargo build --workspace
```

**Try the CLI** — validate an example file right away:

```bash
cargo run -p tree-doc-cli -- validate examples/story.tree.json
cargo run -p tree-doc-cli -- view examples/story.tree.json
```

**Try the browser viewer** — build the WASM module and open the drag-and-drop HTML page:

```bash
cargo build -p tree-doc-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web target/wasm32-unknown-unknown/release/tree_doc_wasm.wasm --out-dir web/pkg
cd web && python3 -m http.server 8080
```

Then open http://localhost:8080 and drag any `.tree.json` file onto the page.

---

## CLI Usage

The CLI binary is called `tree-doc` and has three commands.

### `validate` — Check a document for errors

Runs JSON Schema validation followed by five graph integrity checks. Exits with code 0 if valid, 1 if errors found.

```bash
cargo run -p tree-doc-cli -- validate examples/minimal.tree.json
```

```
✓ examples/minimal.tree.json is valid (3 nodes, 2 edges, tier 0)
```

Validating an invalid file shows precise diagnostics:

```bash
cargo run -p tree-doc-cli -- validate examples/invalid/trunk-cycle.tree.json
```

```
✗ examples/invalid/trunk-cycle.tree.json has validation errors
  error [trunk-cycle]: Trunk path contains a cycle: n1 -> n2 -> n3
    at path: n1 -> n2 -> n3
  warning [general-cycle]: Cycle detected among 3 nodes: n3, n2, n1
    at path: n3 -> n2 -> n1

  1 error, 1 warning
```

### `view` — Walk the trunk path

Renders the trunk (primary reading path) as a linear sequence, showing branch counts at fork points.

```bash
cargo run -p tree-doc-cli -- view examples/story.tree.json
```

```
The Enchanted Garden
────────────────────
7 nodes, 7 edges

[start] You discover a hidden gate in the garden wall. Ivy curls around iron bars.
  ├── [trunk] -> enter
  └── +1 branch
      · Climb the wall

[enter] You push the gate open and step inside. A fountain glistens at the center.
  ├── [trunk] -> fountain
  └── +1 branch
      · Wander the paths

[fountain] You approach the fountain. A coin glints at the bottom of the clear water.
  ├── [trunk] -> wish

[wish] You toss a coin into the fountain and make a wish. The water shimmers.
  ├── [trunk] -> ending

[ending] The garden seems to respond to your presence. You feel at peace.
  └── (end of trunk)
```

### `info` — Document summary

Displays node count, edge count, trunk length, branch count, tier level, and validity.

```bash
cargo run -p tree-doc-cli -- info examples/story.tree.json
```

```
examples/story.tree.json
────────────────────────
  Tier:            1
  Nodes:           7
  Edges:           7
  Trunk length:    4
  Branches:        3
  Valid:           yes
```

## Running All Examples

Try each example to see how the validator and viewer handle different documents:

```bash
# Valid documents
cargo run -p tree-doc-cli -- validate examples/minimal.tree.json
cargo run -p tree-doc-cli -- validate examples/story.tree.json
cargo run -p tree-doc-cli -- validate examples/empty-document.tree.json

# Invalid documents — each triggers a different validation rule
cargo run -p tree-doc-cli -- validate examples/invalid/missing-fields.tree.json
cargo run -p tree-doc-cli -- validate examples/invalid/duplicate-ids.tree.json
cargo run -p tree-doc-cli -- validate examples/invalid/dangling-edge.tree.json
cargo run -p tree-doc-cli -- validate examples/invalid/trunk-cycle.tree.json
cargo run -p tree-doc-cli -- validate examples/invalid/general-cycle.tree.json
cargo run -p tree-doc-cli -- validate examples/invalid/orphan-node.tree.json

# View the trunk path of each valid document
cargo run -p tree-doc-cli -- view examples/minimal.tree.json
cargo run -p tree-doc-cli -- view examples/story.tree.json
cargo run -p tree-doc-cli -- view examples/empty-document.tree.json

# Summary info
cargo run -p tree-doc-cli -- info examples/minimal.tree.json
cargo run -p tree-doc-cli -- info examples/story.tree.json
```

## Validation Rules

The validator checks documents in two passes.

**Pass 1 — Schema validation** ensures the JSON structure matches the format (required fields, correct types). Uses an embedded JSON Schema (Draft 2020-12).

**Pass 2 — Semantic validation** checks graph integrity:

| Rule | Severity | What it checks |
|------|----------|----------------|
| `duplicate-node-id` | Error | No two nodes share the same `id` |
| `dangling-edge` | Error | Every edge's `source` and `target` reference an existing node |
| `trunk-cycle` | Error | The trunk path (following `isTrunk` edges from root) does not loop |
| `general-cycle` | Warning | Strongly connected components in the full graph (cycles are valid for dialogue loops, but worth noting) |
| `orphan-node` | Advisory | Every node is reachable from the root via edges |

Errors make the document invalid (exit code 1). Warnings and advisories are informational.

## Format Tiers

The Tree Document Format has three tiers of complexity:

- **Tier 0** — Minimal: `formatVersion`, `rootNodeId`, `nodes`, `edges`
- **Tier 1** — Adds `minReaderVersion`, `features`, and document-level `metadata` (title, author, etc.)
- **Tier 2** — Multi-tree documents with `trees` map and cross-tree references *(not yet implemented)*

The validator auto-detects the tier and reports it in the output.

## Browser Viewer (WASM)

The project includes a **standalone HTML page** (`web/index.html`) that runs entirely client-side — no backend needed. It loads a WASM build of the core library and provides a drag-and-drop interface for `.tree.json` files.

When you drop a file, the viewer shows three panels:

- **Validation** — pass/fail badge with full diagnostic listing (errors in red, warnings in yellow, advisories in blue)
- **Document Info** — tier, node count, edge count, trunk length, branch count
- **Trunk View** — the full trunk path rendered step-by-step with branch badges and labels

The UI uses a dark theme styled after GitHub's dark mode.

### Build and run

**Step 1** — Build the WASM module:

```bash
cargo build -p tree-doc-wasm --target wasm32-unknown-unknown --release
```

**Step 2** — Generate JavaScript bindings:

```bash
wasm-bindgen --target web target/wasm32-unknown-unknown/release/tree_doc_wasm.wasm --out-dir web/pkg
```

**Step 3** — Serve the `web/` directory (any static file server works):

```bash
cd web
python3 -m http.server 8080
```

**Step 4** — Open http://localhost:8080 in Chrome, Firefox, or Safari and drag a `.tree.json` file onto the page. Try it with any of the files in the `examples/` directory.

### WASM prerequisites

If you haven't set up the WASM toolchain yet:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

## Tests

Run the full test suite:

```bash
cargo test --workspace
```

This runs 46 tests covering:

- JSON parsing and serde roundtrips
- Schema validation (valid files pass, invalid files fail)
- All five semantic validation rules with dedicated test cases
- Trunk viewer traversal with various document shapes
- Edge cases: empty documents, self-loops, extra fields, missing trunk markers, multiple trunk edges

## Project Structure

```
tree-document-format/
├── crates/
│   ├── tree-doc-core/       Core library (types, parsing, validation, viewer)
│   ├── tree-doc-cli/        CLI binary (validate, view, info commands)
│   └── tree-doc-wasm/       WASM bindings for browser use
├── schemas/
│   ├── tier0.schema.json    JSON Schema (Draft 2020-12) for Tier 0
│   └── tier1.schema.json    JSON Schema (Draft 2020-12) for Tier 1
├── examples/                Valid and invalid example documents
└── web/                     Standalone HTML/CSS/JS browser viewer
```

## Using the Library

The `tree-doc-core` crate can be used as a library in your own Rust project:

```rust
use tree_doc_core::{validate_document, parse, build_trunk_view};

// Validate a document
let json = std::fs::read_to_string("my-document.tree.json")?;
let result = validate_document(&json)?;

if result.is_valid {
    println!("Valid! {} nodes, {} edges", result.stats.node_count, result.stats.edge_count);
} else {
    for error in &result.errors {
        eprintln!("{}", error);
    }
}

// Build a trunk view
let doc = parse(&json)?;
let view = build_trunk_view(&doc)?;
for step in &view.steps {
    println!("[{}] {}", step.node_id, step.content);
}
```

## Roadmap

- [x] JSON Schemas (Tier 0 + Tier 1)
- [x] Core library (types, parsing, schema + semantic validation, trunk viewer)
- [x] CLI (`validate`, `view`, `info`)
- [x] WASM browser viewer
- [x] Example documents (valid + invalid)
- [ ] Tier 2 multi-tree support
- [ ] Published crate on crates.io
- [ ] Stable 1.0 format spec

## License

MIT
