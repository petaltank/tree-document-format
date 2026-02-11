use std::path::PathBuf;

use colored::Colorize;
use tree_doc_core::error::ValidationResult;
use tree_doc_core::viewer::TrunkView;

pub fn print_validation_result(result: &ValidationResult, file: &PathBuf) {
    if result.is_valid {
        println!(
            "{} {} is valid ({} nodes, {} edges, tier {})",
            "✓".green().bold(),
            file.display(),
            result.stats.node_count,
            result.stats.edge_count,
            result.stats.tier,
        );
    } else {
        println!(
            "{} {} has validation errors",
            "✗".red().bold(),
            file.display(),
        );
    }

    for diag in &result.errors {
        println!(
            "  {} {}: {}",
            "error".red().bold(),
            format!("[{}]", diag.rule).dimmed(),
            diag.message,
        );
        println!("    {} {}", "at".dimmed(), diag.location);
    }

    for diag in &result.warnings {
        println!(
            "  {} {}: {}",
            "warning".yellow().bold(),
            format!("[{}]", diag.rule).dimmed(),
            diag.message,
        );
        println!("    {} {}", "at".dimmed(), diag.location);
    }

    for diag in &result.advisories {
        println!(
            "  {} {}: {}",
            "advisory".blue().bold(),
            format!("[{}]", diag.rule).dimmed(),
            diag.message,
        );
        println!("    {} {}", "at".dimmed(), diag.location);
    }

    // Summary line
    let error_count = result.errors.len();
    let warning_count = result.warnings.len();
    let advisory_count = result.advisories.len();
    let total = error_count + warning_count + advisory_count;
    if total > 0 {
        println!();
        let mut parts = Vec::new();
        if error_count > 0 {
            parts.push(format!("{} error{}", error_count, if error_count == 1 { "" } else { "s" }));
        }
        if warning_count > 0 {
            parts.push(format!("{} warning{}", warning_count, if warning_count == 1 { "" } else { "s" }));
        }
        if advisory_count > 0 {
            parts.push(format!("{} advisor{}", advisory_count, if advisory_count == 1 { "y" } else { "ies" }));
        }
        println!("  {}", parts.join(", "));
    }
}

pub fn print_trunk_view(view: &TrunkView) {
    println!("{}", view.title.bold());
    println!("{}", "─".repeat(view.title.len()).dimmed());
    println!("{}", view.stats.dimmed());
    println!();

    for (i, step) in view.steps.iter().enumerate() {
        // Node header
        println!("{} {}", format!("[{}]", step.node_id).cyan(), step.content);

        if step.is_terminal {
            println!("  {} {}", "└──".dimmed(), "(end of trunk)".dimmed());
        } else if let Some(ref target) = step.trunk_target {
            println!(
                "  {} {} {}",
                "├──".dimmed(),
                "[trunk]".green(),
                format!("-> {target}").dimmed(),
            );
        }

        if step.branch_count > 0 {
            let badge = format!("+{} branch{}", step.branch_count, if step.branch_count == 1 { "" } else { "es" });
            println!("  {} {}", "└──".dimmed(), badge.yellow());
            for label in &step.branch_labels {
                println!("      {} {}", "·".dimmed(), label);
            }
        }

        if i < view.steps.len() - 1 {
            println!();
        }
    }
}

pub fn print_info(result: &ValidationResult, file: &PathBuf) {
    let stats = &result.stats;
    println!("{}", file.display().to_string().bold());
    println!("{}", "─".repeat(file.display().to_string().len()).dimmed());
    println!("  {:<16} {}", "Tier:".dimmed(), stats.tier);
    println!("  {:<16} {}", "Nodes:".dimmed(), stats.node_count);
    println!("  {:<16} {}", "Edges:".dimmed(), stats.edge_count);
    println!("  {:<16} {}", "Trunk length:".dimmed(), stats.trunk_length);
    println!("  {:<16} {}", "Branches:".dimmed(), stats.branch_count);
    println!(
        "  {:<16} {}",
        "Valid:".dimmed(),
        if result.is_valid {
            "yes".green().to_string()
        } else {
            "no".red().to_string()
        }
    );
}
