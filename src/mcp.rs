//! This module contains the MCP implementation.

use std::borrow::BorrowMut;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

use crate::tools::*;
use crate::utils::*;

type State<T> = Arc<Mutex<T>>;

pub async fn send_prompt(prompt: &str, model: &str) {
    let state = State::new(Mutex::new(String::from("Thinking")));

    // Cycling loading text is printed in parrallel
    let loading = thread::spawn({
        let state = Arc::clone(&state);
        move || {
            let mut points = 0;
            loop {
                let state_message = state.lock().unwrap().clone();
                print!("\r{}{}                    ", state_message, ".".repeat(points));
                let _ = std::io::stdout().flush();
                thread::sleep(Duration::from_millis(300));

                // Checking if main thread is done
                if state.lock().unwrap().is_empty() {
                    break;
                }
                points = (points + 1) % 4;
            }
        }
    });

    let manager_response = Ollama::default()
        .generate(GenerationRequest::new(model.to_string(), prompt))
        .await;

    {
        // Limited scope to manage the lock
        let mut state_ = state.lock().unwrap();
        state_.clear();
        state_.push_str("Working in progress");
    }
    
    match manager_response {
        Ok(answer) => {
            let answer = answer.response;
            
            send_agent(answer, model, Arc::clone(&state)).await;

            let mut state = state.lock().unwrap();
            **state.borrow_mut() = String::new();       
            drop(state);
            
        },
        Err(error) => {
            println!("\rFailed to get a response from ollama : {error}")
        }
    }

    let _ = loading.join();
}

async fn send_agent(answer: String, model_name: &str, state: Arc<Mutex<String>>) {
    let lines = answer.split("\n").collect::<Vec<&str>>();
    let agent_name = lines[0];

    match agent_name {
        "File_System" => {
            let vec = lines[1].split(" ").collect::<Vec<&str>>();
            if let (Some(path), Some(action)) = (vec.get(0), vec.get(1)) {
                let content = lines[2..lines.len()].iter().map(|s| s.to_string()).collect::<String>();
                match action_files(*path, *action, content) {
                    Ok(file_content) => {
                        let input = lines[2..lines.len()].iter().map(|s| *s).collect::<String>();
                        let prompt = get_prompt(input, agent_name);
                        let prompt = prompt.replace("{file_content}",&file_content);
                        let agent_response = Ollama::default()
                            .generate(GenerationRequest::new(model_name.to_string(), prompt))
                            .await;
                        println!("\r{}",unwrap_response(agent_response));
                    },
                    Err(message) => {
                        println!("\r{}",message);
                    }
                }
            }
            else {
                println!("\rThe Manager is tired, try to be more concise.");
            }
        },
        "Sqlite3" => {
            let input = lines[1..lines.len()].iter().map(|s| s.to_string()).collect::<String>();
            let prompt = get_prompt(input, agent_name);
            let agent_response = Ollama::default()
                .generate(GenerationRequest::new(model_name.to_string(), prompt))
                .await;
            query_sqlite3(unwrap_response(agent_response)).await;
        }
        _ => {
            let input = lines[1..lines.len()].iter().map(|s| s.to_string()).collect::<String>();
            let prompt = get_prompt(input, agent_name);
            let agent_response = Ollama::default()
                .generate(GenerationRequest::new(model_name.to_string(), prompt))
                .await;
            println!("\r{}",unwrap_response(agent_response));
        }
    }
}
