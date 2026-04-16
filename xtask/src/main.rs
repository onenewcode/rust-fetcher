use std::{env, process};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: cargo xtask <command> [args]");
        eprintln!("Available commands: im, live, hook");
        process::exit(1);
    }

    match args[0].as_str() {
        "live" | "im" => {
            let status = process::Command::new("cargo")
                .args(["run", "--package", "cli", "--"])
                .args(&args)
                .status()?;
            if !status.success() {
                process::exit(status.code().unwrap_or(1));
            }
        }
        "hook" => {
            run_hook()?;
        }
        _ => {
            eprintln!("Unknown task: {}", args[0]);
            eprintln!("Available commands: im, live, hook");
            process::exit(1);
        }
    }

    Ok(())
}

fn run_hook() -> anyhow::Result<()> {
    // 1. Cargo Check
    println!("Running cargo check...");
    let status = process::Command::new("cargo")
        .args(["check", "--all-targets", "--all-features"])
        .status()?;
    if !status.success() {
        print_deny("compilation_failed", "Cargo check failed.");
        process::exit(0); // Exit 0 to let agent read JSON
    }

    // 2. Cargo Fmt
    println!("Running cargo fmt...");
    let _ = process::Command::new("cargo")
        .args(["fmt", "--all"])
        .status()?;

    // 3. Cargo Clippy Fix
    println!("Running cargo clippy --fix...");
    let _ = process::Command::new("cargo")
        .args([
            "clippy",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--all-targets",
            "--all-features",
        ])
        .status()?;

    // 4. Final Clippy Check
    println!("Running final clippy check...");
    let status = process::Command::new("cargo")
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()?;

    if !status.success() {
        print_deny("clippy_warnings_remain", "Clippy warnings remain.");
        process::exit(0);
    }

    println!("{}", serde_json::json!({"decision": "proceed"}));
    Ok(())
}

fn print_deny(code: &str, reason: &str) {
    let response = serde_json::json!({
        "decision": "deny",
        "reason": format!("Hook rejected session due to {code}: {reason}\n\nINSTRUCTION: Please fix the reported issues and try again.")
    });
    println!("{}", response);
}
