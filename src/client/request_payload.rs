use serde::{Serialize, Deserialize};

// macro will automatically write the extra code to convert this struct into JSON bytes using serde crate
// structure for individual chat messages
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String, 
    pub content: String
}

// struct for extra body options
#[derive(Serialize)]
pub struct ChatTemplateKwargs {
    pub enable_thinking: bool,
}

#[derive(Serialize)]
pub struct NvidiaRequest {
    pub model: String, 
    pub messages: Vec<Message>,
    pub temperature: f32, 
    pub top_p: f32,
    pub max_tokens: u32,
    pub chat_template_kwargs: ChatTemplateKwargs,
    pub reasoning_budget: u32, 
    pub stream: bool,
}


