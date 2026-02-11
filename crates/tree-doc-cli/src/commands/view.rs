use std::path::PathBuf;
use std::process;

use crate::output;

pub fn run(file: &PathBuf) {
    let json_str = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {e}", file.display());
            process::exit(2);
        }
    };

    // Validate first
    let result = match tree_doc_core::validate_document(&json_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parsing '{}': {e}", file.display());
            process::exit(2);
        }
    };

    if !result.is_valid {
        output::print_validation_result(&result, file);
        eprintln!("\nDocument has errors. Fix them before viewing.");
        process::exit(1);
    }

    // Parse and build trunk view
    let doc = match tree_doc_core::parse(&json_str) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing '{}': {e}", file.display());
            process::exit(2);
        }
    };

    let view = match tree_doc_core::build_trunk_view(&doc) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error building trunk view: {e}");
            process::exit(2);
        }
    };

    output::print_trunk_view(&view);
}
