// tells rust the agent module exists
mod agent;
mod client;
mod filesystem;

pub mod config;
pub mod error;

use config::Config;

#[tokio::main]
async fn main() {
    let config = match Config::load() {
        Ok (c) => c,
        Err (e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    // match filesystem::process_patch("sandbox.txt", "world", "\norchestrator works perfectly").await {
    //     Ok(()) => println!("Patch processed successfully"), 
    //     Err(e) => {
    //         eprintln!("Patch not successful {}", e);
    //     }
    // }
    
    let mut agent = agent::Agent::new(config.api_key);
    match agent.run().await {
        Ok(()) => println!("\n\nAgent concluded"),
        Err(e) => println!("{}", e)
    }
}
