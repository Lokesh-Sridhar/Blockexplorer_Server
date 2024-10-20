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

async fn handle_refresh_block_request() -> Result<impl warp::Reply, warp::Rejection> {
    let _ = rpc_functions::load_data().await;
    Ok(warp::reply::with_status("Refresh completed", warp::http::StatusCode::OK))
}

fn with_graph(
    graph: Arc<Graph>,
) -> impl Filter<Extract = (Arc<Graph>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || graph.clone())
}

async fn handle_block_request(
    block_height: i64,
    graph: Arc<Graph>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Query Neo4j to get block data by height
    let query = neo4rs::query("
        MATCH (b:Block { height: $height })
        RETURN b.hash AS hash, b.size AS size, b.time AS time
    ")
    .param("height", block_height);

    let mut result = graph.execute(query).await.unwrap();

    if let Some(row) = result.next().await.unwrap() {
        let block_hash: String = row.get("hash").unwrap();
        let block_size: i64 = row.get("size").unwrap();
        let block_time: String = row.get("time").unwrap();

        let block_data = BlockData {
            height: block_height,
            hash: block_hash,
            size: block_size,
            time: block_time,
        };

        // Return block data as JSON
        Ok(warp::reply::json(&block_data))
    } else {
        Err(warp::reject::not_found())
    }
}


async fn handle_transaction_request(
    txid: String,
    graph: Arc<Graph>,
    origin: Option<String>,  // Capture the Origin header from the request
) -> Result<impl warp::Reply, warp::Rejection> 

{
    println!("Transaction ID: {}", txid);
    
    let query = neo4rs::query(
        "
        MATCH (t:Transaction { txid: $txid })
        RETURN t.txid AS txid, t.height AS height
        "
        ).param("txid", txid as String);

    let mut result = graph.execute(query).await.unwrap();

    if let Some(row) = result.next().await.unwrap() {

        let transaction_id: String = row.get("txid").unwrap();
        let transaction_height: i64 = row.get("height").unwrap();

        let transaction_data = TransactionData {
            txid: transaction_id,
            height: transaction_height,
        };

        let allowed_origin = origin.unwrap_or_else(|| "*".to_string()); // Default to * if no Origin is provided

        // Return transaction data as JSON with explicit CORS header
        Ok(warp::reply::with_header(
            warp::reply::json(&transaction_data),
            "Access-Control-Allow-Origin",
            HeaderValue::from_str(&allowed_origin).unwrap(),
        ))
    } else {
        Err(warp::reject::not_found())
    }
}