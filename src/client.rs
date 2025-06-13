//! The Client just permite the communication between the UI and the Agegents.

use std::sync::{Arc,Mutex};
use std::borrow::BorrowMut;
use std::thread;
use std::time::Duration;

use crate::utils::State;
use crate::agent::launch_agent;

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
            let agent_msg = &state_agent.lock().unwrap().0;
            let ui_msg = &mut state.lock().unwrap().0;
            ui_msg.clear();
            ui_msg.push_str(&format!("{}{}                    ",agent_msg, ".".repeat(points)));
        }

        thread::sleep(Duration::from_millis(300));
        points = (points + 1) % 4;
    }

    let _ = agent_thread.join();

    {
        // We get the current state of the agent and send it to the UI !
        let agent_msg = &state_agent.lock().unwrap().0;
        let mut ui_msg = state.lock().unwrap();
        **ui_msg.borrow_mut() = (agent_msg.to_string(), false);
    }

    let state_agent = state_agent.lock().unwrap();
    drop(state_agent);
}

// TODO : add a counter to display the number of seconds