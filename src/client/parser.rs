use crate::{client::response_payload, error::HarnessError};

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

            if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.as_ref()) {
                extracted_tokens.push_str(content);
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
