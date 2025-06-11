use std::borrow::BorrowMut;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

use sqlite::Value;

fn prompt(input: String, agent_name: &str) -> String {
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
            send_prompt(&prompt("".to_string(), "Sqlite3"), &model).await;
        }
    }
}

async fn send_prompt(prompt: &str, model: &str) {
    let done = State::new(Mutex::new(false));

    // Cycling loading text is printed in parrallel
    let loading = thread::spawn({
        let done = Arc::clone(&done);
        move || {
            let mut points = 0;
            loop {
                print!("\rThinking{}                    ", ".".repeat(points));
                let _ = std::io::stdout().flush();
                thread::sleep(Duration::from_millis(300));

                // Checking if main thread is done
                if **done.lock().unwrap().borrow_mut() {
                    break;
                }
                points = (points + 1) % 4;
            }
        }
    });

    // Waiting for prompt answer
    let ollama = Ollama::default();
    let res = ollama
        .generate(GenerationRequest::new(model.to_string(), prompt))
        .await;

    // Notifying loading thread that we are done
    let mut done = done.lock().unwrap();
    **done.borrow_mut() = true;
    drop(done);

    let _ = loading.join();

    match res {
        Ok(res) => {
            // Executing prompt's response on database
            let con = sqlite::open("res/clients.db");
            match con {
                Err(e) => println!("\rCould not connect to database : {e}"),
                Ok(con) => {
                    // Sanitizing prompt's response
                    let query = res
                        .response
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
        Err(e) =>
        // Sometimes, shit happens
        {
            println!("\rFailed to get a response from ollama : {e}")
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
