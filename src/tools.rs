//! This module contains the tools of the MCP.

use std::{fs::OpenOptions, path::PathBuf};
use std::io::{Read, Write};

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

pub fn action_files(path: &str, action: &str, content: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    let action = action.to_uppercase();
    let action = action.as_str();

    match action {
        "WRITE" => {
            match OpenOptions::new().read(true).write(true).truncate(true).create(true).open(&path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Err(format!("Successfully write into {} !",path.display())),
                        Err(error) => Err(format!("Error when try to write in {}\n\t{}",path.display(),error))
                    }
                }
                Err(error) => {
                    Err(format!("Error : with the following file : {}\n\t{}",path.display(),error))
                }
            }
        },
        "APPEND" => {
            match OpenOptions::new().read(true).append(true).open(&path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => Err(format!("Successfully append content into {} !",path.display())),
                        Err(error) => Err(format!("Error when try to append content in {}\n\t{}",path.display(),error))
                    }
                }
                Err(error) => {
                    Err(format!("Error : with the following file : {}\n\t{}",path.display(),error))
                }
            }
        }
        "READ" | _ => {
            match OpenOptions::new().read(true).open(&path) {
                Ok(mut file) => {
                    let mut result = String::new();
                    let _ = file.read_to_string(&mut result);
                    Ok(result)
                }
                Err(error) => {
                    Err(format!("Error : with the following file : {}\n\t{}",path.display(),error))
                }
            }
        },
    }
}
