//! This module contains the tools of the MCP.

use std::{fs::OpenOptions, path::PathBuf};
use std::io::{Read, Write};
use std::process::Command;

use mylog::error;
use sqlite::Value;

pub async fn query_sqlite3(query: String) -> Result<String, String> {
    let con = sqlite::open("res/clients.db");
    match con {
        Err(e) => Err(format!("\rCould not connect to database : {e}")),
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
                Err(e) => Err(format!("Error while executing query : {e}")),
                Ok(cursor) => {
                    let mut result = String::new();
                    // Result handling (just printing for now)
                    for (i, row) in cursor.into_iter().enumerate() {
                        match row {
                            Err(e) => println!("Error for row {i} : {e}"),
                            Ok(row) => {
                                // Nice formatting for user's pleasure
                                for field in row.iter() {
                                    result.push_str(&format!("{}, ", extract_value(field.1)));
                                }
                            }
                        }
                    }
                    Ok(result)
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

pub fn bash_command(agent_command: String) -> Result<String, String> {
    if agent_command.contains("sudo") {
        return Err(format!("Can't execute the following command due to permission : {}", agent_command))
    }

    let args = shell_words::split(&agent_command)
        .map_err(|e| format!("{}",e))?;
    let (command, args) = args.split_first().unwrap();

    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("{}",e))?;

    if output.status.success() {
        let msg = String::from_utf8_lossy(&output.stdout).to_string();
        if !msg.is_empty() {
            Ok(msg)
        }
        else {
            Ok(format!("Successfully run the following comand :\n`{}`", agent_command))
        }
    }
    else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
