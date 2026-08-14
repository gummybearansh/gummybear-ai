pub mod request_payload;
pub mod response_payload;
pub mod parser;

use crate::{error::HarnessError};
use request_payload::{NvidiaRequest, Message, ChatTemplateKwargs};

pub async fn send_request(api_key: &str, messages: Vec<Message>) -> Result<reqwest::Response, HarnessError> {
    let client = reqwest::Client::new();
    let url = "https://integrate.api.nvidia.com/v1/chat/completions";

    // build the payload using our structs 
    let payload = NvidiaRequest {
        // model: "nvidia/nemotron-3.5-lightning-30b-a3b".to_string(),
        model: "nvidia/nemotron-3-ultra-550b-a55b".to_string(),
        messages: messages,
        temperature: 1.0, 
        top_p: 0.95,
        max_tokens: 100000, 
        chat_template_kwargs: ChatTemplateKwargs {
            enable_thinking: true,
        },
        reasoning_budget: 1000,
        stream: true,
    };

    println!("Sending post request to NVIDIA...");

    // sending the request 
    // if fails will just return error to caller
    let response = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    // Catch API errors (like Bad Request or Unauthorized) immediately
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(HarnessError::UnknownTool(format!("NVIDIA API Error: {}", error_text))); // Temporary error reuse for debugging
    }
    
    Ok(response)
}

