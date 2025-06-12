mod tools;
mod mcp;
mod utils;

use std::io::Write;
use mcp::*;
use utils::*;

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
