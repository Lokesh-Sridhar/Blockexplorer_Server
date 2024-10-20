use bitcoincore_rpc::bitcoin::absolute::Height;
use serde::Serialize;
use neo4rs::*;
use warp::Filter;
use std::sync::Arc;
use crate::{graph_functions, rpc_functions};
use warp::http::HeaderValue;
use dotenv::dotenv;
use std::env;


#[derive(Serialize)]
struct BlockData {
    height: i64,
    hash: String,
    size: i64,
    time: String,
}

#[derive(Serialize)]
struct TransactionData {
    txid: String,
    height: i64,
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let graph: Arc<Graph> = graph_functions::get_graph().await?;

    // Define the route to handle block requests
    let blocks_route = warp::path!("blocks" / i64)        
                        .and(with_graph(graph.clone())) // Inject the graph instance
                        .and_then(handle_block_request); // Call the handler directly

    let refresh_block_route = warp::path!("blocks" / "refresh")
                                    .and_then(handle_refresh_block_request);
    
    let transactions_route = warp::path!("transactions" / String)        
                        .and(with_graph(graph.clone())) // Inject the graph instance
                        .and(warp::header::optional::<String>("origin"))  // Extract the Origin header
                        .and_then(handle_transaction_request);
    
    // Create a CORS filter
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["Content-Type"]); // Allow specific HTTP methods
    
    // Apply the CORS filter to your route
    let routes = blocks_route.or(refresh_block_route).or(transactions_route).with(cors);

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], port.parse().unwrap())).await;

    Ok(())
}