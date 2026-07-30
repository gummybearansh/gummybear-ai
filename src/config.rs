use crate::error::HarnessError;

pub struct Config {
    pub api_key: String,
    pub url: String, 
    pub model: String,
}

impl Config {
    pub fn load () -> Result<Self, HarnessError>{
        // loads the .env into memory
        dotenvy::dotenv().ok();

        let conf =  Config {
            api_key: std::env::var("NVIDIA_API_KEY")?,
            url: "https://integrate.api.nvidia.com/v1/chat/completions".to_string(), // need String type not &str - so dynamic alloctiation using to_string
            model: "nvidia/nemotron-3-ultra-550b-a55b".to_string(),
        };

        Ok(conf)
    }
}
