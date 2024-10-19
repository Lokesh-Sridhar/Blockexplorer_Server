extern crate bitcoincore_rpc;
use tokio;

mod rpc_functions;
mod graph_functions;
mod webserver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Loading data from Bitcoin Core
    // Starting the web server
    webserver::start_server().await?;

    Ok(())
}
