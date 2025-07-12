# Data Station
Data Station is a *simulacrum* of an ***MCP server*** with ***AI Agents***. The aim of this project is to test the limits of AI agent architectures in performing the same tasks as MCP servers.

The project is designed to run **locally** for **security** and **privacy** reasons, which is why it uses only small AI models (between **1B** and **4B** parameters).

The architecture relies on a few AI agents to break down complex tasks and use tools effectively. When the user sends a prompt, a *Manager* agent decomposes it into subtasks to be handled by other agents (*LLM_Core*, *Execute_Command*, *Sqlite3*). These agents access specific tools and their sole purpose is to generate inputs for built-in functions. This is how we transform a simple *LLM* chat into a *simulacrum* of an ***MCP server***.


## Dependencies
Please have ollama up and running with any LLM model installed. You can choose what model to use with the `/model [your model]` command. Default is `gemma3:latest`.

## How to run ?
Start by running the `res/load_data.py` to load exemple data into `res/clients.db`. To execute the main program, execute the `cargo run` command.

## How to use ?
Type `/help` in the program prompt to have a list of commands.

## Future work :
|State|Ideas|
|:-:|:-:|
|✅|Reduce latency by using smaller models<br> ***gemma3:4b*** into ***gemma3:1b***|
|🚧​|Improving the prompts for the Agents|
|🚧​|Implement a real server|
|🚧​|Adding an *LLM Guard* to prevent from issues|
|🚧​|Adding an Agent for formatting|
