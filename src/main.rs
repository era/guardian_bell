use clap::Parser;
use std::path::PathBuf;

mod admin;
mod app;
mod metrics;
mod server;
mod wal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    grpc_server_port: u16,
    #[arg(short, long)]
    log_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), app::AppError> {
    let args = Args::parse();
    app::App::run_server(args.grpc_server_port, &args.log_path).await
}
