use crate::client::parser;
use crate::filesystem::process_patch;
use crate::{client::send_request, filesystem::read_file};
use crate::client::request_payload::Message;
use crate::{client::parser::{UniversalParser, parse_text_stream}, error::HarnessError};
use futures_util::StreamExt;
use std::io::Write; // trait to be in scope so that i can use the flush() method it implements


pub struct Agent {
    api_key: String, 
    conversation_history: Vec<Message>
}

impl Agent {
    // constructor 
    pub fn new(api_key: String) -> Self {
        Self {
            api_key, 
            conversation_history: vec![
                Message {
                    role: "system".to_string(),
                    content: "
You are an autonomous agent with local filesystem access. You interact with the system by outputting specific XML tags.

AVAILABLE TOOLS:
1. READ: <READ>file_path</READ>
2. PATCH: <PATCH>file_path|||exact_search_string|||new_replace_string</PATCH>

RULES:
- You must use the exact '|||' delimiter for the PATCH tool.
- Do not output any XML tags other than these. 
- Wait for the user (the system) to return the tool output before continuing.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Say 'Hello'. Read 'sandbox.txt'. Then, use the PATCH tool to replace the contents of the file with a nice message. Finally, say 'Goodbye'.".to_string(),
                }
            ]        
        }
    }

    // engine 
    pub async fn run(&mut self) -> Result<(), HarnessError> {
        loop {
            let response = send_request(&self.api_key, self.conversation_history.clone()).await?;
            // get the stream of bytes from the response 
            let mut stream = response.bytes_stream();
            let mut parser = UniversalParser::new();

            let mut current_assistant_message = String::new();
            let mut tool_triggered = false;

            // loop over the chunks asynchronously 
            // loop label to tell which loop to exit out of 
            'stream: while let Some(chunk_result) = stream.next().await {
                // get the bytes
                let bytes = chunk_result?;
                // print the raw chunks as they arrive 
                let content  = std::str::from_utf8(&bytes)?; 

                // parse this streamed congtent and print the extracted tokens in real time (freeing the buffer) 
                if let Some(text) = parse_text_stream(content)? {
                    // feed the tokens to our state machine and get the evnts 
                    let events = parser.push_token(&text);

                    // consume the events 
                    for event in events {
                        match event {
                            parser::StreamEvent::Printable(safe_text) => {
                                print!("{}", safe_text);
                                std::io::stdout().flush()?;
                                current_assistant_message.push_str(&safe_text); // add it to the current agent message so it remembers it
                            }
                            parser::StreamEvent::ToolTrigger { name, payload } => {
                                // the parser intercepted an event
                                println!("\n\n[⚙️ TOOL INTERCEPTED] Name: {}, Payload: {}\n", name, payload);
                                current_assistant_message.push_str(&format!("<{}>{}</{}>", name, payload, name)); 
                                // remember the tool it called and add it to history 
                                self.conversation_history.push(Message{
                                    role: "assistant".to_string(),
                                    content: current_assistant_message.clone()
                                });

                                let tool_response = execute_tool(&name, &payload).await?;
                                // add the tool response to history as well
                                self.conversation_history.push(Message { 
                                    role: "user".to_string(), 
                                    content: tool_response 
                                });

                                tool_triggered = true;
                                break 'stream; // explicitly break out of the while loop
                            }
                        }
                    }
                }

            }
            // exit condition
            if !tool_triggered {
                // if we reached here - stream finished naturally without ending on a tool call 
                // safe final message to history and exitprogram 
                self.conversation_history.push(Message { 
                    role: "assistant".to_string(), 
                    content: current_assistant_message
                });

                return Ok(());
            }

            // if a tool is triggered we do nothing
            // outer loop automatically spins again
        }
    }
}

pub async fn execute_tool (name: &str, payload: &str) -> Result<String, HarnessError>{
    match name {
        "READ" => read_file(payload).await,
        "PATCH" => {
            // 3 arguments in payload 
            // file_path|||search_string|||replace_string
            let parts: Vec<&str> = payload.split("|||").collect();

            // validate LLM partitioned it correctly 
            if parts.len() != 3 {
                return Ok(format!("Invalid PATCH format, expected 3 parts, separated by |||, `file_path|||search_string|||replace_string`, found {} parts", parts.len()));
            }

            let file_path = parts[0];
            let search_string = parts[1];
            let replace_string = parts[2];

            process_patch(file_path, search_string, replace_string).await?;

            Ok(format!("Successfully patched {}, changes have been written to disk", file_path))
        },
        _ => Err(HarnessError::UnknownTool(name.to_string()))
    }
}
