mod tools;
mod utils;
mod client;
mod agent;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use std::io;
use client::*;
use utils::*;
use mylog::{error};

#[tokio::main]
async fn main() {
    ui().await;
}

async fn ui() {
    println!(
        "Welcome to Data Station v0.1 !\nStart prompting or type `/help` for a list of commands."
    );

    loop {
        let mut model = "gemma3:latest".to_string();

        print!("\r>>> ");
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
                    Some(arg) => {
                        model.clear();
                        model.push_str(arg);
                    },
                    None => println!("Current model: {model}"),
                },
                _ => println!(
                    "Unknown command : {}.\nType '/help' for a list of commands.",
                    command[0]
                ),
            }
        } else {
            let (mut tx, mut rx) = tokio::sync::mpsc::channel(100);

            let client_thread = tokio::spawn({
                async move || {
                    client(input, model, tx).await;
                }
            } ());

            let mut run = true;
            while run {
                //let _ = tokio::time::sleep(Duration::from_millis(10)).await;

                match rx.recv().await {
                    Some(State::Update(message)) => {
                        print!("\r{}", message)
                    },
                    Some(State::Done(message)) => {
                        print!("\r");
                        println!("{}", message);
                        run = false
                    },
                    None => {
                        error!("The receiver receive nothing...");
                    }
                }
            }

            let _ = client_thread.await;
            drop(rx);
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
