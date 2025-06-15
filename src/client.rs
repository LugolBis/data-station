//! The Client just permite the communication between the UI and the Agegents.

use std::{sync::{Arc,Mutex}, time::Duration};
use tokio;
use crate::tokio::sync::mpsc::Sender;
use std::io::Write;

use crate::utils::State;
use crate::agent::launch_agent;
use mylog::{error};

pub async fn client(prompt: String, model_name: String, ui_tx: Sender<State>) {
    let (mut agent_tx, mut agent_rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn({
        async move || {
            launch_agent(prompt.as_str(), model_name.as_str(), agent_tx).await;
        }
    } ());

    let mut dots = 0usize;
    let mut run = true;
    let mut current_msg = String::new();
    while run {  
        match agent_rx.recv().await {
            Some(State::Update(msg)) => {
                current_msg.clear();
                current_msg.push_str(&msg);
                let msg = format!("{}{}          ", msg.clone(), ".".repeat(dots));

                if let Err(error) = ui_tx.send(State::Update(msg)).await {
                    error!("Sender client -x-> UI : {}", error)
                }
            },
            Some(State::Done(msg)) => {
                if let Err(error) = ui_tx.send(State::Done(msg)).await {
                    error!("Sender client -x-> UI : {}", error)
                }
                else {
                    run = false // We stop the client only if the UI receive the msg
                }
            },
            None => {
                let msg = format!("{}{}          ", current_msg.clone(), ".".repeat(dots));
                if let Err(error) = ui_tx.send(State::Update(msg)).await {
                    error!("Sender client -x-> UI : {}", error)
                }
            }
        }

        dots = (dots + 1) % 4;
    }
}

// TODO : add a counter to display the number of seconds