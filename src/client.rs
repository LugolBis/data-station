//! The Client just permite the communication between the UI and the Agegents.

use std::sync::{Arc,Mutex};
use std::borrow::BorrowMut;
use std::thread;
use std::time::Duration;

use crate::utils::State;
use crate::agent::launch_agent;
use mylog::{error};

pub async fn client(prompt: String, model_name: String, state: State<String>) {
    let state_agent = State::new(Mutex::new((String::new(), true)));

    let agent_thread = thread::spawn({
        let state_agent_ = Arc::clone(&state_agent);
        async move || {
            launch_agent(prompt.as_str(), model_name.as_str(), state_agent_).await;
        }
    });

    let mut points = 0usize;
    while state_agent.lock().unwrap().1 {
        {
            // We get the current state of the agent and send it to the UI !
            if let Ok(agent_msg) = state_agent.lock() {
                if let Ok(ui_msg) = &mut state.lock() {
                    ui_msg.0 = agent_msg.0.clone();
                }
                else {
                    error!("Can't get mut the ui_msg.")
                }
            }
            else {
                error!("Can't get the agent_msg.")
            }
        }

        thread::sleep(Duration::from_millis(300));
        points = (points + 1) % 4;
    }

    {
        // We get the current state of the agent and send it to the UI !
        if let Ok(agent_msg) = &state_agent.lock() {
            if let Ok(ui_msg) = &mut state.lock() {
                ui_msg.0 = agent_msg.0.clone();
                ui_msg.1 = false;
            }
            else {
                error!("Can't get mut the ui_msg.")
            }
        }
        else {
            error!("Can't get the agent_msg.")
        }
    }

    let _ = agent_thread.join();
    drop(state_agent);
}

// TODO : add a counter to display the number of seconds