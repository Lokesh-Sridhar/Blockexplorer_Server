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

