use std::borrow::BorrowMut;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

use ollama_rs::Ollama;
use ollama_rs::error::OllamaError;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationResponse};

use crate::tools::*;
use crate::utils::*;

pub async fn launch_agent(prompt: &str, model: &str, state_agent: State<String>) {

    let manager_response = Ollama::default()
        .generate(GenerationRequest::new(model.to_string(), prompt))
        .await;

    {
        // Limited scope to manage the lock
        let client_msg = &mut state_agent.lock().unwrap().0;
        client_msg.clear();
        client_msg.push_str("The Manager Agent has completed his task.  |   Step : 1");
    }
    
    match manager_response {
        Ok(answer) => {
            let answer = answer.response;
            let tasks = answer.split("---------")
                .into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
            
            // This will be used to store the result of each previous step.
            let mut answer = String::new();
            let mut step = 2usize;

            for task in tasks {
                let lines = task.split("\n")
                    .into_iter().collect::<Vec<&str>>();

                let agent_name = lines[0];
                let input = lines[1..lines.len()-1].iter()
                    .map(|s| s.to_string()).collect::<String>();
                let need_previous_step = lines[lines.len()-1].to_uppercase();

                let mut prompt = get_prompt(input, agent_name);
                if need_previous_step.contains("YES") && !answer.is_empty() {
                    prompt.push_str(&format!("**Data that you need**:\n{}",answer));
                }

                let agent_response = Ollama::default()
                    .generate(GenerationRequest::new(model.to_string(), prompt))
                    .await;

                match agent_response {
                    Ok(agent_answer) => {
                        match call_tool(agent_name, agent_answer.response).await {
                            Ok(answer_tool) => {
                                {
                                    let client_msg = &mut state_agent.lock().unwrap().0;
                                    client_msg.clear();
                                    client_msg.push_str(&format!("The {} Agent has completed his task.  |   Step : {}",
                                    agent_name, step));
                                }
        
                                answer.clear();
                                answer.push_str(&answer_tool);
                                step += 1;
                            },
                            Err(error_tool) => {
                                {
                                    let error_msg = format!("ERROR : The {} Agent hasn't completed his task.  |   Step : {}\nFailed to use his tool : {}",
                                        agent_name, step, error_tool);
                                    let mut client_msg = state_agent.lock().unwrap();
                                    **client_msg.borrow_mut() = (error_msg, false);
                                }
                            }
                        }
                    },
                    Err(error) => {
                        {
                            let error_msg = format!("ERROR : The {} Agent hasn't completed his task.  |   Step : {}\nFailed to get a response from ollama : {}",
                                agent_name, step, error);
                            let mut client_msg = state_agent.lock().unwrap();
                            **client_msg.borrow_mut() = (error_msg, false);
                        }
                    }
                }
            }

            {
                let result = format!("{}", answer);
                let mut client_msg = state_agent.lock().unwrap();
                **client_msg.borrow_mut() = (result, false);
            }
        },
        Err(error) => {
            {
                let error_msg = format!("ERROR : The Manager Agent hasn't completed his task.\nFailed to get a response from ollama : {}", error);
                let mut client_msg = state_agent.lock().unwrap();
                **client_msg.borrow_mut() = (error_msg, false);
            }
        }
    }
}

async fn call_tool(agent_name: &str, agent_prompt: String) -> Result<String, String> {
    match agent_name {
        "Sqlite3" => {
            query_sqlite3(agent_prompt).await
        },
        "File_System" => {
            let lines = agent_prompt.split("\n").into_iter().collect::<Vec<&str>>();
            let path = lines[0];
            let mut action = String::new();
            let action_ = lines[1].to_uppercase();
            for action_type in ["READ","WRITE","APPEND"] {
                if action_.contains(action_type) {
                    action.push_str(action_type);
                    break;
                }
            }
            let content = lines[2..lines.len()].into_iter()
                .map(|s| s.to_string()).collect::<String>();
            action_files(path, action.as_str(), content)
        },
        _ => {
            Err(format!("The following {} Agent hasn't access to a tool.", agent_name))
        }
    }
}