mod request_payload;
mod response_payload;
mod parser;

use crate::{client::parser::parse_text_stream, error::HarnessError};
use request_payload::{NvidiaRequest, Message, ChatTemplateKwargs};
use futures_util::StreamExt;
use std::io::Write; // trait to be in scope so that i can use the flush() method it implements

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
                content: "Hello, write a short poem about a gummy bear, in 100 words".to_string(),
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
        let content  = std::str::from_utf8(&bytes)?; 

        // parse this streamed congtent and print the extracted tokens in real time (freeing the buffer) 
        if let Some(text) = parse_text_stream(content)? {
            print!("{}", text);
            std::io::stdout().flush()?;
        }

    }
    // need to return () wrapped in Ok (function signature)
    Ok(())
}

