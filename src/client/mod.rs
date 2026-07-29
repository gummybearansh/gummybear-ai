mod request_payload;

use request_payload::{NvidiaRequest, Message, ChatTemplateKwargs};

pub async fn call_nvidia(api_key: &str){
    println!("Client received api key starting with {}", &api_key[..5]);

    let client = reqwest::Client::new();
    let url = "https://integrate.api.nvidia.com/v1/chat/completions";

    // build the payload using our structs 
    let payload = NvidiaRequest {
        model: "nvidia/nemotron-3-ultra-550b-a55b".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, write a short poem about a gummy bear.".to_string(),
            },
        ],
        temperature: 1.0, 
        top_p: 0.95,
        max_tokens: 100000, 
        chat_template_kwargs: ChatTemplateKwargs {
            enable_thinking: true,
        },
        reasoning_budget: 1000,
        stream: false,
    };

    println!("Sending post request to NVIDIA...");

    // sending the request 
    let response = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;
    
    match response {
        Ok(res) => {
            println!("Response status {}", res.status());

            // grab the body asynchronously 
            let body = res.text().await.unwrap_or_default();
            println!("body: {}", body);
        }
        Err(e) => println!("Request failed {}", e)
    }

} 

