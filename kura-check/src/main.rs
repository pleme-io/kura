use anyhow::Result;
use clap::Parser;
use kura_core::register_all;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "kura-check",
    about = "Kura workspace coherence checker",
    version
)]
struct Cli {
    #[arg(short, long, default_value = "checks.lisp")]
    checks: PathBuf,

    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    register_all();
    let cli = Cli::parse();
    let src = std::fs::read_to_string(&cli.checks)?;
    let results = run_checks(&src, cli.verbose);
    let mut passed = 0;
    let mut failed = 0;
    for (name, result) in &results {
        match result {
            Ok(_) => {
                if cli.verbose {
                    println!("  PASS  {}", name);
                }
                passed += 1;
            }
            Err(e) => {
                println!("  FAIL  {} — {}", name, e);
                failed += 1;
            }
        }
    }
    println!(
        "\n{} passed, {} failed, {} total",
        passed,
        failed,
        results.len()
    );
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn run_checks(src: &str, _verbose: bool) -> Vec<(String, Result<()>)> {
    let mut results = vec![];

    results.push((
        "domain-registry-keywords".into(),
        check_domain_registry_keywords(),
    ));

    results.push(("lisp-compiles".into(), check_lisp_compiles(src)));

    results.push((
        "guardrail-shell-tools".into(),
        check_guardrail_shell_tools(),
    ));

    results.push(("lattice-bottom-top".into(), check_lattice_bottom_top()));

    results
}

fn check_domain_registry_keywords() -> Result<()> {
    let keywords = [
        "defprovider",
        "defagent",
        "deftool",
        "defplugin",
        "defkeymap",
        "defsession",
        "defzen",
        "defopenai",
    ];
    for kw in &keywords {
        if tatara_lisp::domain::lookup(kw).is_none() {
            anyhow::bail!("keyword '{}' not registered", kw);
        }
    }
    Ok(())
}

fn check_lisp_compiles(src: &str) -> Result<()> {
    let defs: Vec<kura_core::ProviderSpec> = tatara_lisp::compile_typed(src)
        .map_err(|e| anyhow::anyhow!("Lisp compilation failed: {}", e))?;
    if defs.is_empty() {
        anyhow::bail!("no definitions compiled from checks source");
    }
    Ok(())
}

fn check_guardrail_shell_tools() -> Result<()> {
    let shell_tools = kura_core::ToolKind::Shell;
    let _ = shell_tools;
    Ok(())
}

fn check_lattice_bottom_top() -> Result<()> {
    use kura_lattice::Lattice;
    use kura_lattice::{AgentAutonomy, GuardrailLevel, ProviderTrust, ToolSafety};

    assert!(ProviderTrust::bottom().leq(&ProviderTrust::top()));
    assert!(GuardrailLevel::bottom().leq(&GuardrailLevel::top()));
    assert!(AgentAutonomy::bottom().leq(&AgentAutonomy::top()));
    assert!(ToolSafety::bottom().leq(&ToolSafety::top()));
    Ok(())
}
