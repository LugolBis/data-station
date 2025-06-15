use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use mylog::{error,info};
use crate::tokio::sync::mpsc::Sender;

use crate::tools::*;
use crate::utils::*;

pub async fn launch_agent(prompt: &str, model: &str, client_tx: Sender<State>) {

    if let Err(error) = client_tx.send(State::Update("Thinking".to_string())).await {
        error!("Sender agent -x-> client : {}", error);
    }

    let prompt = get_prompt(prompt.to_string(), "Manager");
    let manager_response = Ollama::default()
        .generate(GenerationRequest::new(model.to_string(), prompt))
        .await;

    if let Err(error) = client_tx.send(State::Update("The Manager Agent has completed his task.  |   Step : 1".to_string())).await {
        error!("Sender agent -x-> client : {}", error);
    }
    
    match manager_response {
        Ok(answer) => {
            let answer = answer.response;
            info!("Manager response :\n{}\n\n",answer);
            let tasks = parse_tasks(answer);
            
            // This will be used to store the result of each previous step.
            let mut answer = String::new();
            let mut step = 2usize;

            for task in tasks {
                let mut agent_name = String::new();
                let mut need_previous_step = false;
                let mut task_prompt = String::new();

                if let Ok(task_infos) = parse_task(task) {
                    agent_name.push_str(&task_infos.0);
                    need_previous_step = task_infos.1;
                    task_prompt.push_str(&task_infos.2);
                }
                else {
                    let error_msg = format!("Failed when try to parse an intermediate task. The last answer is :\n\n{}",answer);
                    if let Err(error) = client_tx.send(State::Done(error_msg)).await {
                        error!("Sender agent -x-> client : {}", error);
                    }
                    return;
                }

                let mut prompt = get_prompt(task_prompt, &agent_name);
                if need_previous_step && !answer.is_empty() {
                    prompt.push_str(&format!("\n[Data you need]\n{}",answer));
                }

                let agent_response = Ollama::default()
                    .generate(GenerationRequest::new(model.to_string(), prompt))
                    .await;

                match agent_response {
                    Ok(agent_answer) => {
                        info!("{} response :\n{}\n\n", agent_name, agent_answer.response);
                        answer.push_str(&agent_answer.response);
                        match call_tool(&agent_name, agent_answer.response).await {
                            Ok(answer_tool) => {
                                let msg = format!("The {} Agent has completed his task.  |   Step : {}", agent_name, step);
                                if let Err(error) = client_tx.send(State::Update(msg)).await {
                                    error!("Sender agent -x-> client : {}", error);
                                }
        
                                answer.clear();
                                answer.push_str(&answer_tool);
                                step += 1;
                            },
                            Err(_) => {
                                let error_msg = format!("The following answer could be wrong, paid attention :\n{}", answer);
                                if let Err(error) = client_tx.send(State::Done(error_msg)).await {
                                    error!("Sender agent -x-> client : {}", error);
                                }
                                return;
                            }
                        }
                    },
                    Err(error) => {
                        let error_msg = format!("ERROR : The {} Agent hasn't completed his task.  |   Step : {}\nFailed to get a response from ollama : {}",
                            agent_name, step, error);
                        if let Err(error) = client_tx.send(State::Done(error_msg)).await {
                            error!("Sender agent -x-> client : {}", error);
                        }
                        return;
                    }
                }
            }

            if let Err(error) = client_tx.send(State::Done(answer)).await {
                error!("Sender agent -x-> client : {}", error);
            }
            return;
        },
        Err(error) => {
            let error_msg = format!("ERROR : The Manager Agent hasn't completed his task.\nFailed to get a response from ollama : {}", error);
            if let Err(error) = client_tx.send(State::Done(error_msg)).await {
                error!("Sender agent -x-> client : {}", error);
            }
            return;
        }
    }
}

async fn call_tool(agent_name: &str, agent_prompt: String) -> Result<String, String> {
    match agent_name {
        "Bash" => {
            bash_command(agent_prompt)
        },
        "Sqlite3" => {
            query_sqlite3(agent_prompt).await
        },
        "LLM_Core" => {
            Ok(get_prompt(agent_prompt, "LLM_Core"))
        },
        _ => {
            error!("Unknow agent : {}",agent_name);
            Err("Catch an unknow Agent, who hasn't access to a tool.".to_string())
        }
    }
}