mod request_payload;
mod response_payload;

use crate::error::HarnessError;
use request_payload::{NvidiaRequest, Message, ChatTemplateKwargs};
use futures_util::StreamExt;

pub async fn call_nvidia(api_key: &str) -> Result<(), HarnessError>{
    println!("Client received api key starting with {}", &api_key[..5]);

    let client = reqwest::Client::new();
    let url = "https://integrate.api.nvidia.com/v1/chat/completions";

    // build the payload using our structs 
    let payload = NvidiaRequest {
        model: "nvidia/nemotron-3-ultra-550b-a55b".to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: "Hello, write a short poem about a gummy bear, in 1000 words".to_string(),
            },
        ],
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
    

    // get the stream of bytes from the response 
    let mut stream = response.bytes_stream();

    // loop over the chunks asynchronously 
    while let Some(chunk_result) = stream.next().await {
        // get the bytes
        let bytes = chunk_result?;
        // print the raw chunks as they arrive 
        let text  = std::str::from_utf8(&bytes)?; 
        // chunks could have multiple SSE lines together separated by new line 
        for line in text.lines() {
            let line = line.trim();
            
            // skip if line is empty or the final [DONE] signal 
            if line.is_empty() || line == "data: [DONE]"{
                continue;
            }

            // extract json from the line after "data: " (normal SSE structure)
            if let Some(json_str) = line.strip_prefix("data: "){
                // deserialize into our struct 
                if let Ok(chunk) = serde_json::from_str::<response_payload::ChatCompletionChunk>(json_str) {
                    // if content exists - print it without the new line 
                    if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.as_ref()){
                        print!("{}", content);

                        // flush stdout so tokens don't sit buffered in terminal 
                        use std::io::{self, Write};
                        let _ = io::stdout().flush();
                    }
                }
            }
        }
    }
    // need to return () wrapped in Ok (function signature)
    Ok(())
}

