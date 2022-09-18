use my_redis::{DEFAULT_PORT,server};

use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::signal;
#[tokio::main]
pub async fn main() -> my_redis::Result<()>{
    tracing_subscriber::fmt::try_init()?;
    let cli = Cli::from_args();
    let port = cli.port.as_deref().unwrap_or(DEFAULT_PORT);
    let listenr = TcpListener::bind(format!("127.0.0.1:{}",port)).await?;
    server::run(listenr,signal::ctrl_c()).await;
    Ok(())
}

#[derive(Debug,StructOpt)]
#[structopt(name="my-redis-server")]
struct Cli {
    #[structopt(name="port",long="--port")]
    port:Option<String>
}