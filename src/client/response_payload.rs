use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk{
    pub choices: Vec<ChoiceChunk>
}

#[derive(Debug, Deserialize)]
pub struct ChoiceChunk{
    pub delta: DeltaChunk
}

#[derive(Debug, Deserialize)]
pub struct DeltaChunk {
    pub content: Option<String>
}
