use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cli-executor")]
#[command(about = "A CLI executor with middleware support", long_about = None)]
pub struct Cli {
    /// Protocol to use (http, websocket, etc.)
    #[arg(value_enum)]
    pub protocol: Protocol,

    /// Target URL or address
    pub target: String,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Request payload (JSON or raw)
    #[arg(short, long)]
    pub payload: Option<String>,

    /// Request headers (key=value pairs)
    #[arg(short = 'H', long, value_parser = parse_header)]
    pub headers: Option<Vec<(String, String)>>,

    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// Number of retries
    #[arg(short = 'r', long, default_value = "0")]
    pub retry_count: u32,

    /// Path to external middleware executables
    #[arg(short, long)]
    pub middleware: Option<Vec<PathBuf>>,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Execute middleware only (dry run)
    #[arg(long, action = ArgAction::SetTrue)]
    pub dry_run: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Protocol {
    Http,
    Https,
    WebSocket,
    Ws,
    Wss,
    Tcp,
    Udp,
}

fn parse_header(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err("Header must be in format key=value".to_string())
    }
}