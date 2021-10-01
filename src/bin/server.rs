use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::signal;
use my_redis::server;
use my_redis::DEFAULT_PORT;

#[tokio::main]
pub async fn main() -> my_redis::Result<()>{
    tracing_subscriber::fmt::try_init()?;
    let cli = Cli::from_args();
    let port = cli.port.as_deref().unwrap_or(DEFAULT_PORT);
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
    server::run(listener,signal::ctrl_c()).await;
    Ok(())
}
#[derive(StructOpt,Debug)]
#[structopt(name = "my-redis-server", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"), about = "A Redis server")]
struct Cli {
    port: Option<String>,
}