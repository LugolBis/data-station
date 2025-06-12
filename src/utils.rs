use std::fs;
use ollama_rs::error::OllamaError;
use ollama_rs::generation::completion::GenerationResponse;

pub fn get_prompt(input: String, agent_name: &str) -> String {
    let path = format!("agents/{}.txt", agent_name);
    if let Ok(context) = fs::read_to_string(&path) {
        context.replace("{user_prompt}", &input)
    }
    else {
        "{user_prompt}".to_string()
    }
}

pub fn unwrap_response(response:  Result<GenerationResponse, OllamaError>) -> String {
    match response {
        Ok(answer) => {
            answer.response
        },
        Err(error) => {
            format!("\rFailed to get a response from ollama : {error}")
        }
    }
}
