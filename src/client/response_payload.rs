use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk{
    pub choices: Option<Vec<ChoiceChunk>>
}

#[derive(Debug, Deserialize)]
pub struct ChoiceChunk{
    pub delta: DeltaChunk
}

#[derive(Debug, Deserialize)]
pub struct DeltaChunk {
    // pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
}
