use std::borrow::BorrowMut;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

use sqlite::Value;

fn get_prompt(input: String, agent_name: &str) -> String {
    let path = format!("agents/{}.txt", agent_name);
    if let Ok(context) = fs::read_to_string(&path) {
        context.replace("{user_prompt}", &input)
    }
    else {
        "{user_prompt}".to_string()
    }
}

type State<T> = Arc<Mutex<T>>;

#[tokio::main]
async fn main() {
    println!(
        "Welcome to Data Station v0.1 !\nStart prompting or type `/help` for a list of commands."
    );

    let mut model = "gemma3:latest".to_string();

    loop {
        print!(">>> ");
        let _ = std::io::stdout().flush();

        // Reading user input
        let mut input = String::new();
        if let Err(_) = std::io::stdin().read_line(&mut input) {
            println!("An error occured while reading input. Please try again.");
            continue;
        }
        input.pop(); // Removes '\n' from input

        if input.is_empty() {
            continue;
        }

        // Checking if user typed a command
        if input.starts_with('/') {
            let command: Vec<&str> = input.split(' ').collect();
            match command[0] {
                "/exit" | "/quit" => break,
                "/help" => print_help(),
                "/model" => match command.get(1) {
                    Some(arg) => model = arg.to_string(),
                    None => println!("Current model: {model}"),
                },
                _ => println!(
                    "Unknown command : {}.\nType '/help' for a list of commands.",
                    command[0]
                ),
            }
        } else {
            send_prompt(&get_prompt(input, "Manager"), &model).await;
        }
    }
}

async fn send_prompt(prompt: &str, model: &str) {
    let done = State::new(Mutex::new(String::from("Thinking")));

    // Cycling loading text is printed in parrallel
    let loading = thread::spawn({
        let done = Arc::clone(&done);
        move || {
            let mut points = 0;
            loop {
                let state = done.lock().unwrap().clone();
                print!("\r{}{}                    ", state, ".".repeat(points));
                let _ = std::io::stdout().flush();
                thread::sleep(Duration::from_millis(300));

                // Checking if main thread is done
                if done.lock().unwrap().is_empty() {
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
        let mut done_ = done.lock().unwrap();
        done_.clear();
        done_.push_str("Working in progress");
    }
    
    match manager_response {
        Ok(answer) => {
            let answer = answer.response;
            let vec = answer.split("\n").collect::<Vec<&str>>();
            let agent_name = vec[0];
            let input = vec[1..vec.len()].iter()
                .map(|s| s.to_string())
                .collect::<String>();

            let prompt = get_prompt(input, agent_name);
            let response = Ollama::default()
                .generate(GenerationRequest::new(model.to_string(), prompt))
                .await;

            let mut done = done.lock().unwrap();
            **done.borrow_mut() = String::new();       
            drop(done);

            match (response, agent_name) {
                (Ok(answer),"Sqlite3") => {
                    query_sqlite3(answer.response).await;
                },
                (Ok(answer),_) => {
                    println!("\n{}",answer.response);
                },
                (Err(error),_) => {
                    println!("\rFailed to get a response from ollama : {error}");
                }
            }
        },
        Err(error) => {
            println!("\rFailed to get a response from ollama : {error}")
        }
    }

    let _ = loading.join();
}

async fn query_sqlite3(query: String) {
    let con = sqlite::open("res/clients.db");
    match con {
        Err(e) => println!("\rCould not connect to database : {e}"),
        Ok(con) => {
            // Sanitizing prompt's response
            let query = query
                .split('\n')
                .filter(|x| x.len() < 3 || &x[..3] != "```")
                .map(|x| x.to_string())
                .reduce(|x, y| format!("{x}\n{y}"))
                .unwrap();
            println!("\r                                \nSending SQL : {query}");

            // Database execution
            let cursor = con.prepare(&query);
            match cursor {
                Err(e) => println!("Error while executing query : {e}"),
                Ok(cursor) => {
                    // Result handling (just printing for now)
                    for (i, row) in cursor.into_iter().enumerate() {
                        match row {
                            Err(e) => println!("Error for row {i} : {e}"),
                            Ok(row) => {
                                // Nice formatting for user's pleasure
                                for field in row.iter() {
                                    print!("{}, ", extract_value(field.1));
                                }
                                println!();
                            }
                        }
                    }

                    println!("\rDone !");
                }
            }
        }
    }
}

/// Quick parsing function extracting the
/// text representation of a `sqlite3::Value`
/// without its debug informations.
fn extract_value(value: &Value) -> String {
    // Start of from the debug format
    let d = format!("{value:?}");
    let mut result = String::new();

    let mut parenthesis_level = 0;

    // Parsing
    for c in d.chars() {
        if c == ')' {
            parenthesis_level -= 1;
        }
        if parenthesis_level >= 1 {
            result.push(c);
        }
        if c == '(' {
            parenthesis_level += 1;
        }
    }

    result
}

/// Prints the help text which contains
/// a list of commands and their purpose.
/// Please don't extract this as a constant
/// (even though it looks like we should)
/// because it I plan to make this help
/// text reactive to available commands.
fn print_help() {
    // TODO: Make help reactive
    println!(
        "== Data Station v0.1 commands ==\n\
         /help ....... Displays this text\n\
         /exit or /quit ............ Exits program\n\
         /model [model] ... Switch model\n\
            \t No `model` prints current\n"
    );
}
