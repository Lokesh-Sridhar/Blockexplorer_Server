use neo4rs::Query;
use tokio;
use neo4rs::*;
use std::sync::Arc;
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;

use crate::graph_functions;
extern crate bitcoincore_rpc;

async fn get_rpc_data(function_name: &str, params: &[serde_json::Value]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from a .env file if present
    let rpc_url = std::env::var("BITCOIN_RPC_URL")?;
    let rpc_user = std::env::var("BITCOIN_RPC_USER")?;
    let rpc_password = std::env::var("BITCOIN_RPC_PASS")?;

    let client = Client::new();

    let payload = json!({
        "jsonrpc": "1.0",
        "method": function_name,
        "params": params,
        "id": "1"
    });

    let response_result = client.post(&rpc_url)
        .basic_auth(rpc_user, Some(rpc_password))
        .json(&payload)
        .send()
        .await;

    match response_result {
        Ok(response) => {
            if response.status().is_success() {
                let response_json: serde_json::Value = response.json().await?;
                Ok(response_json)
            } else {
                println!("Request failed with status: {}", response.status());
                Err(Box::from("Request failed"))
            }
        }
        Err(e) => {
            println!("Error making request: {:?}", e);
            Err(Box::from(e))
        }
    }
}


pub async fn load_data() -> Result<(), Box<dyn std::error::Error>> {

    tokio::spawn(async {
        let block_count_value = get_rpc_data("getblockcount", &[]).await.unwrap();
        let block_count: u64 = block_count_value["result"].as_u64().unwrap();
        println!("Block count: {}", block_count);

        // Fetch the latest block
        let best_block_hash_value = get_rpc_data("getbestblockhash", &[]).await.unwrap();
        let best_block_hash_str = best_block_hash_value["result"].as_str().unwrap();
        let block_json = get_rpc_data("getblock", &[best_block_hash_str.into()]).await.unwrap();

        // Connect to the Neo4j database
        let graph = graph_functions::get_graph().await.unwrap();

        // Fetch block details from Bitcoin Core
        let transaction_size = block_json["result"]["nTx"].as_i64().unwrap();
        let time = block_json["result"]["time"].as_i64().unwrap();
        load_block(transaction_size, block_count, best_block_hash_str, time, &graph).await.unwrap();

        // Fetch transactions for the block
        let tx_arr = block_json["result"]["tx"].as_array().unwrap();
        load_transactions_for_block(tx_arr, block_count, &graph).await.unwrap();
    });
    
    Ok(())
}