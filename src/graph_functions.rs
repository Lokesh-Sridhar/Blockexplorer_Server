use neo4rs::*;
use std::sync::Arc;

use dotenv::dotenv;
use std::env;

pub async fn get_graph() -> Result<Arc<Graph>, Box<dyn std::error::Error>> {
    dotenv().ok(); // Load environment variables from a .env file if present

    // Connect to Neo4j
    let config = ConfigBuilder::default()
        .uri(&env::var("NEO4J_URI")?)
        .user(&env::var("NEO4J_USER")?)
        .password(&env::var("NEO4J_PASSWORD")?)
        .build()?;
    let graph = Arc::new(Graph::connect(config).await?);
    Ok(graph)
}