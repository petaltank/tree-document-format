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

    let result = match tree_doc_core::validate_document(&json_str) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parsing '{}': {e}", file.display());
            process::exit(2);
        }
    };

    output::print_info(&result, file);
}
