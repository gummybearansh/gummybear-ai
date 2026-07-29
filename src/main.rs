// tells rust the agent module exists
mod agent;
mod client;

#[tokio::main]
async fn main() {
    // loads the .env into memory
    dotenvy::dotenv().ok();

    // fetch the variable 
    let api_key = std::env::var("NVIDIA_API_KEY");
    let key = match api_key {
        Ok(key) => key,
        Err(_e) => panic!("could not load the api key")
    };

    client::call_nvidia(&key).await;

}
