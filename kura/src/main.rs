use anyhow::Result;
use clap::{Parser, Subcommand};
use kura_agent::{Conversation, SessionStore};
use kura_core::register_all;
use kura_provider::ProviderRouter;
use kura_run::{parse_file, DagExecutor};
use kura_tool::ToolExecutor;
use kura_tui::{App, KuraTheme, TuiEventStream, detect_capabilities};

#[derive(Parser)]
#[command(
    name = "kura",
    about = "Kura — the Rust+Lisp agentic coding harness, Ghostty-native",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "default")]
    session: String,

    #[arg(short, long)]
    model: Option<String>,

    #[arg(short = 'P', long)]
    provider: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Launch {
        #[arg(short, long)]
        agent: Option<String>,
    },
    Sessions,
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    Check,
    Run {
        #[arg(short, long)]
        file: String,
    },
}

#[derive(Subcommand)]
enum ConfigActions {
    Show,
    Edit,
    Reload,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_stderr();
    register_all();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Launch { agent }) => run_tui(agent, cli.model, cli.provider).await,
        Some(Commands::Sessions) => list_sessions().await,
        Some(Commands::Config { action }) => handle_config(action).await,
        Some(Commands::Check) => run_checks().await,
        Some(Commands::Run { file }) => run_dag(&file).await,
        None => run_tui(None, cli.model, cli.provider).await,
    }
}

async fn run_tui(
    agent_name: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> Result<()> {
    let capabilities = detect_capabilities();

    let spec = kura_core::AgentSpec {
        name: agent_name.unwrap_or_else(|| "coder".to_string()),
        kind: kura_core::AgentKind::Coder,
        system_prompt: Some("You are a helpful coding assistant running inside kura.".to_string()),
        tools: vec![],
        plugins: vec![],
        provider,
        model,
        max_turns: Some(50),
        auto_approve: false,
        thinking_budget: None,
    };

    let zen_spec = kura_core::ProviderSpec {
        name: "zen".to_string(),
        kind: kura_core::ProviderKind::Zen,
        base_url: Some("https://opencode.ai/zen/v1".to_string()),
        api_key_env: Some("OPENCODE_API_KEY".to_string()),
        model: Some("opencode/claude-sonnet-4-20250514".to_string()),
        priority: 10,
        max_tokens: None,
        temperature: None,
        disabled: false,
    };

    let router = ProviderRouter::new(&[zen_spec]);
    let tools = ToolExecutor::new();
    let theme = KuraTheme::nord();
    let capabilities = detect_capabilities();
    let mut app = App::new(theme, &spec, router, capabilities.clone());

    app.setup_ghostty();

    let _guard = kura_ghostty::TerminalRestoreGuard::new()?;

    let mut event_stream = crossterm::event::EventStream::new();
    let mut tui_events = TuiEventStream::new(capabilities, std::time::Duration::from_millis(100));

    let mut conversation = Conversation::new(spec.system_prompt.clone());

    while app.running {
        if let Some(tui_event) = tui_events.next_event(&mut event_stream).await {
            if let Some(action) = app.handle_event(tui_event) {
                match action {
                    kura_tui::app::AppAction::SubmitInput(text) => {
                        conversation.add_user_message(text);
                    }
                    kura_tui::app::AppAction::Quit => {
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    app.teardown_ghostty();
    Ok(())
}

async fn list_sessions() -> Result<()> {
    let store = SessionStore::new(SessionStore::default_dir());
    let sessions = store.list().await?;
    if sessions.is_empty() {
        println!("no sessions found");
    } else {
        for id in &sessions {
            println!("{}", id);
        }
    }
    Ok(())
}

async fn handle_config(action: ConfigActions) -> Result<()> {
    match action {
        ConfigActions::Show => {
            println!("kura config — shikumi-powered (YAML/TOML/Lisp/Nix)");
        }
        ConfigActions::Edit => {
            println!("use $EDITOR or configure via nix/blackmatter-kura");
        }
        ConfigActions::Reload => {
            println!("config reload — shikumi hot-reload via ConfigWatcher");
        }
    }
    Ok(())
}

async fn run_checks() -> Result<()> {
    println!("kura checks — domain registry + coherence");
    Ok(())
}

async fn run_dag(file: &str) -> Result<()> {
    let dag = kura_run::parse_file(file)?;
    println!("loading DAG: {}", dag.name);
    
    let executor = kura_run::DagExecutor::new(dag);
    let result = executor.execute().await?;
    
    if result.success {
        println!("DAG completed successfully");
    } else {
        eprintln!("DAG failed:");
        for (node_id, node_result) in &result.node_results {
            if let Some(error) = &node_result.error {
                eprintln!("  {}: {}", node_id.0, error);
            }
        }
    }
    Ok(())
}

fn init_tracing_stderr() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
}
