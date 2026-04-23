use anyhow::Result;
use clap::Parser;
use kura_run::{parse_file, DagExecutor};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "kura-run")]
#[command(about = "Run DAG-based prompt orchestration", long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    file: PathBuf,
    
    #[arg(short, long, default_value = "info")]
    verbose: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.verbose))
        .init();
    
    if args.file == PathBuf::from(".") {
        anyhow::bail!("please specify a DAG file with -f <file>");
    }
    
    let dag = parse_file(&args.file)?;
    tracing::info!("loaded DAG: {}", dag.name);
    
    let executor = DagExecutor::new(dag);
    let result = executor.execute().await?;
    
    if result.success {
        tracing::info!("DAG completed successfully");
    } else {
        tracing::error!("DAG failed");
        for (node_id, node_result) in &result.node_results {
            if let Some(error) = &node_result.error {
                tracing::error!("  {}: {}", node_id.0, error);
            }
        }
    }
    
    Ok(())
}