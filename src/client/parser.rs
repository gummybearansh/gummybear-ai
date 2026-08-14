use crate::{client::response_payload, error::HarnessError};

// state / Label Enum
#[derive(Debug, PartialEq, Clone, Copy)] // PartialEquality - will let me do self.state == ParserState::Chatting
pub enum ParserState {
    Chatting, // LLM talking normally
    ReadingFile, // saw <READ> and are waiting for file path 
    ExecutingCommand, // saw <CMD> and are waiting for terminal command 
    PatchingFile, // saw <Patch> and are waiting for old code 
}

// Event / Output Enum 
#[derive(Debug)]
pub enum StreamEvent {
    // A regular word, main loop prints to the terminal
    Printable(String), // Tuple Variant

    // a finished command, main loop should stop printing and run this function 
    ToolTrigger { name: String, payload: String }, // Struct Variant 
}

// struct to contain these enums 
pub struct UniversalParser {
    pub state: ParserState, 
    pub stream_response: String, // holds partial strings <SE while waiting for ARCH>
    pub payload_buffer: String, // holds actual file paths / code blocks
}

impl UniversalParser {
    // constructor that creates the empty box when program starts
    pub fn new() -> Self {
        Self {
            state: ParserState::Chatting, // always start chatting
            stream_response: String::new(),
            payload_buffer: String::new(),
        }
    }

    // bouncer that handles the streaming tokens and decides what to do with them
    pub fn push_token(&mut self, token: &str) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        // add the incoming chunk from the network
        self.stream_response.push_str(token);

        // infinite loop 
        // keep processing the buffer until it's empty or waiting on the network 
        loop {
            let should_continue = match self.state {
                // if helper function returns true - we loop again, if false we break
                ParserState::Chatting => self.handle_chatting(&mut events),

                // for any of the payload states - we can have one universal helper 
                _ => self.handle_payload(&mut events)
            };

            if !should_continue {
                break;
            }
        }
        events
    }

    // handle normal text and detects OPENING tags
    fn handle_chatting(&mut self, events: &mut Vec<StreamEvent>) -> bool {
        // we can directly add more tool calls here in the future 
        let supported_tags = [
            ("<READ>", ParserState::ReadingFile),
            ("<CMD>", ParserState::ExecutingCommand),
            ("<PATCH>", ParserState::PatchingFile),
        ];

        let mut matched_tag = None;
        let mut earliest_idx = usize::MAX;

        // scan for the earliest occuring tag 
        for &(tag, next_state) in supported_tags.iter() {
            if let Some(idx) = self.stream_response.find(tag) {
                if idx < earliest_idx {
                    earliest_idx = idx;
                    matched_tag = Some((tag, next_state));
                }
            }
        }

        // if we found a tag - slice it out and change the state
        if let Some((tag, next_state)) = matched_tag {
            // print everything before the tag 
            if earliest_idx > 0 {
                let before_tag = self.stream_response[..earliest_idx].to_string();
                events.push(StreamEvent::Printable(before_tag));
            }

            // change the state and slice off the tag 
            self.state = next_state;
            self.stream_response = self.stream_response[earliest_idx + tag.len()..].to_string();

            return true; // tell orchestrator to loop again
        }

        // if no full tag - check if one is forming (<)
        if let Some(start_idx) = self.stream_response.rfind('<') {
            if start_idx > 0 {
                let safe_text = self.stream_response[..start_idx].to_string();
                events.push(StreamEvent::Printable(safe_text));
                self.stream_response = self.stream_response[start_idx..].to_string();
            }
            return false; // tell orchestrator to break and wait for network
        }

        // pure text chat 
        if !self.stream_response.is_empty() {
            events.push(StreamEvent::Printable(self.stream_response.clone()));
            self.stream_response.clear();
        }

        false
    }


    // handle the buffering of data and detects closing tags 
    fn handle_payload(&mut self, events: &mut Vec<StreamEvent>) -> bool {
        // determine which closing tag and tool name we're looking for based on current state 
        let (closing_tag, tool_name) = match self.state {
            ParserState::ReadingFile => ("</READ>", "READ"),
            ParserState::ExecutingCommand => ("</CMD>", "CMD"),
            ParserState::PatchingFile => ("</PATCH>", "PATCH"),
            _ => return false, // chatting is handled separately this is a safe fallback
        };

        // look for the closing tag in the buffer 
        if let Some(end_idx) = self.stream_response.find(closing_tag) {
            // everything before the closing tag is our payload 
            let payload_chunk = self.stream_response[..end_idx].to_string();
            self.payload_buffer.push_str(&payload_chunk);

            // add this tool to be triggered by stream event 
            events.push(StreamEvent::ToolTrigger { 
                name: tool_name.to_string(), 
                payload: self.payload_buffer.clone(),
            });

            // reset our parser back to chatting 
            self.state = ParserState::Chatting;
            self.payload_buffer.clear();

            // slice off the closing tag and keep any leftovers 
            self.stream_response = self.stream_response[end_idx + closing_tag.len()..].to_string();

            return true; // loop again to process leftovers
        }

        // no full closing tags but is one forming? 
        if let Some(end_idx) = self.stream_response.rfind("</") {
            if end_idx > 0 {
                // everything before </ is safe payload data 
                let safe_payload = self.stream_response[..end_idx].to_string();
                self.payload_buffer.push_str(&safe_payload);

                // keep the </ and whatever follows in stream response 
                self.stream_response = self.stream_response[end_idx..].to_string();
            }
            return false; // wait for more network chunks
        }

        // no closing tag at all - entire buffer has payload data
        self.payload_buffer.push_str(&self.stream_response);
        self.stream_response.clear();

        false
    }
}

pub fn parse_text_stream(text: &str) -> Result<Option<String>, HarnessError> {
    // buffer to accumulate all the tokens in this chunk 
    let mut extracted_tokens = String::new();

    for line in text.lines() {
        let line = line.trim();

        // skip control frames 
        if line.is_empty() || line == "data: [DONE]" {
            continue;
        }

        // extract and parse 
        if let Some(json_str) = line.strip_prefix("data: ") {
            let chunk = serde_json::from_str::<response_payload::ChatCompletionChunk>(json_str)?;

            if let Some(choices) = chunk.choices {
                if let Some(choice) = choices.get(0) {
                    // chcek for reasoning
                    if let Some(reasoning) = &choice.delta.reasoning_content {
                        // Let's wrap reasoning in standard terminal ANSI codes to make it GREY/DIM
                        // \x1b[2m starts dim text, \x1b[0m resets it.
                        let formatted_reasoning = format!("\x1b[2m{}\x1b[0m", reasoning);
                        extracted_tokens.push_str(&formatted_reasoning);
                    }

                    //Check for actual speech/XML tags
                    if let Some(content) = &choice.delta.content {
                        extracted_tokens.push_str(content);
                    }
                }
            }
        }
    }


    // return the payload 
    if extracted_tokens.is_empty() {
        Ok(None)
    } else {
        Ok(Some(extracted_tokens))
    }
}
