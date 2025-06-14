use std::fs;
use ollama_rs::error::OllamaError;
use ollama_rs::generation::completion::GenerationResponse;
use std::sync::{Arc, Mutex};

pub type State<String> = Arc<Mutex<(String, bool)>>;

pub fn get_prompt(input: String, agent_name: &str) -> String {
    let path = format!("agents/{}.txt", agent_name);
    if let Ok(context) = fs::read_to_string(&path) {
        context.replace("{user_prompt}", &input)
    }
    else {
        "{user_prompt}".to_string()
    }
}

const AGENTS: [&str;3] = ["LLM_Core", "File_System", "Sqlite3"];

fn parse_agent(agent_name: &str) -> Option<String> {
    for agent in AGENTS {
        if agent_name.contains(agent) {
            return Some(agent.to_string())
        }
    }
    None
}

fn parse_need_previous_step(input: &str) -> bool {
    input.to_uppercase().contains("YES")
}

pub fn parse_task(task: String) -> Result<(String, bool, String), String> {
    let lines = task.split("\n")
        .into_iter().collect::<Vec<&str>>();

    if lines.len() >= 3 {
        if let Some(agent_name) = parse_agent(lines[0]) {
            let need_previous_step = parse_need_previous_step(lines[1]);
            let task_prompt = lines[2..lines.len()].into_iter()
                .map(|s| s.to_string()).collect::<String>();
            Ok((agent_name, need_previous_step, task_prompt))
        }
        else {
            Err("Unknow agent name".to_string())
        }
    }
    else {
        Err("Inconsistant format response from the Manager agent.".to_string())
    }
}
