# Data Station
Data Station is a `simulacrum` of a ***MCP server*** with ***AI Agents***. The aims of this project is to test the limit of the architecture of AI Agents to reponde to the same tasks than MCP servers.
The project is designed to run **locally** for **security** and **privacy**, that why it's only about small AI models here (between **1B**-**4B**).<br><br>
The architecture is based on few AI agents to decompose complexe task and use tools. When the user send a prompt there is an agent *Manager* who's decompose it to be done by the other agents (*LLM_Core*, *Execute_Command*, *Sqlite3*). The other agents can access to specified tools and their task is just to create an input for built-in functions, that how we turn a simple *LLM* chat into a `simulacrum` of ***MCP server***.

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
