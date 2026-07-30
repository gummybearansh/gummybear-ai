// tells rust the agent module exists
mod agent;
mod client;

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

    match client::call_nvidia(&config.api_key).await {
        Ok(_) => println!("\n\nTask finished successfully"), 

        Err (e) => {
            eprintln!("{}", e);

            std::process::exit(1);
        }
    }
}
