# Agent panel

Agent-panel is a control panel and observability platform for optimizing and monitoring LLM/AI agents. The current version focuses on the AI gateway, providing effortless access to 100+ LLMs across 20+ AI platforms. The project is written in Rust.

## Table of contents 
1. [Background](#background)
2. [Supported LLMs](#supported-llms)
3. [Installation, Configure and Run](#installation)
    3.1 [Build](#build)
    3.2 [Configure](#configuration)
    3.3 [Run](#run)
    3.4 [Develop](#develop)
4. [Example Configuration](#example-configuration)
5. [Multi-Platform Configuration](#multi-platform-configuration-and-usage)
6. [Next steps](#roadmap)
7. [Thank you](#thank-you)
8. [Get involved](#get-involved)

## Background
At its core, Agent Panel addresses the complex challenges faced in managing multi-agent systems, particularly those involving numerous function-calling steps and multiple invocations.The project was born out of the difficulties faced while building and optimizing the [Incredible AI agent](https://github.com/IncredibleDevHQ/Incredible.dev). It highlighted the need for detailed observability of the agent's interactions, including sequential and parallel engagements with sub-agents like QnA, code search, code understanding, code generation, and evaluation agents.


The initial version of Agent Panel introduces the AI Gateway, This feature alone significantly reduces the complexity and time required to connect and use different LLMs for various tasks.

Here are the two significant differentiators of the agent panel. 

1. Unlike most observability tools, the agent panel takes an API-first approach rather than the wrapper approach for the OpenAI clients. It makes it easy for Vanilla implementations to integrate. In the future, I'll be building a wrapper for the OpenAI client; until then, it's more like a low-level driver and needs to be integrated using its API.

2. Most observability tools track the LLM calls and their metrics but aren't designed to optimize multi-agent systems. At its core, Agent Panel addresses the complex challenges faced in managing multi-agent systems, particularly those involving numerous function-calling steps and multiple invocations. 

## Supported LLMs

- OpenAI GPT-3.5/GPT-4 (paid, vision, function-calling)
- Gemini: Gemini-1.0/Gemini-1.5 (free, paid, vision, function-calling)
- Claude: Claude-3.5/Claude-3 (vision, paid, function-calling)
- Mistral (paid, function-calling)
- Cohere: Command-R/Command-R+ (paid, function-calling)
- Perplexity: Llama-3/Mixtral (paid)
- Groq: Llama-3/Mixtral/Gemma (free)
- Ollama (free, local, embedding)
- Azure OpenAI (paid, vision, function-calling)
- VertexAI: Gemini-1.0/Gemini-1.5 (paid, vision, function-calling)
- VertexAI-Claude: Claude-3 (paid, vision)
- Bedrock: Llama-3/Claude-3/Mistral (paid, vision)
- Cloudflare (free, paid, vision)
- Replicate (paid)
- Ernie (paid)
- Qianwen (paid, vision, embedding)
- Moonshot (paid)
- ZhipuAI: GLM-3.5/GLM-4 (paid, vision)
- Deepseek (paid)
- Other openAI-compatible platforms

## Installation

### Build 
To install and run the agent-panel project, follow these steps:

1. Clone the repository from GitHub:

```bash
$ git clone https://github.com/IncredibleDevHQ/agent-panel
$ cd agent-panel
```

2. Build the project using Cargo (ensure you have Rust installed on your system):

```bash
cargo build --release
```

3. Create an empty config file called `config.yaml` amd Set the `CONFIG_DIR` environment variable to the directory containing your `config.yaml` file:

```bash
export CONFIG_DIR=/path/to/directory/containing/config.yaml
```


### Configuration

To set up the AI gateway, follow these steps:

1. Open `config.yaml` in a text editor.
2. Start with an empty `clients:` section:

   ```yaml
   clients:
   ```
3. Decide which LLM platforms you want to support. You can include multiple platforms.
4. For each platform you want to support, copy the relevant client configuration from the `example.config.yaml` (provided in the repo's root directory) and paste it under the `clients:` section in your `config.yaml`.

### Example Configuration

Here's an example `config.yaml` that supports both Claude and Gemini:

```yaml
clients:
  - type: claude
    api_key: sk-ant-xxx     # Replace with your actual Claude API key

  - type: gemini
    api_key: xxx            # Replace with your actual Gemini API key
    patches:
      '.*':                                           
        chat_completions_body:
          safetySettings:
            - category: HARM_CATEGORY_HARASSMENT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_HATE_SPEECH
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_SEXUALLY_EXPLICIT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_DANGEROUS_CONTENT
              threshold: BLOCK_NONE
```

Remember to replace the placeholder API keys with your actual credentials.

### Run 

 Run the binary:

```bash
./target/release/agent-panel
```

By default, the server will run on port 8000. If you need to change the port, use the `--port` argument:

```bash
./target/release/agent-panel --port 8080
```

This will start the server on port 8080 instead.

Remember to ensure that your `config.yaml` file is properly set up with the LLM platforms you want to use before running the server.

Once the server is running, you can start making requests to the AI gateway as described in the previous sections.

### Develop 
If you're developing or want to run the project without building a release version, you can use `cargo run`.

```bash
cargo run -- [arguments]
```

For example, to run in development mode with a custom port:

```bash
cargo run -- --port 8080
```

This will compile and run the project in one step, which is useful during development.

## Using the AI Gateway

After configuring your `config.yaml`, you can start making requests to the AI gateway. Here's how to use it:

### Composing HTTP Requests

1. **Endpoint URL**: 
   Send POST requests to `http://127.0.0.1:8000/v1/chat/completions` (adjust if your gateway is configured differently).

2. **Headers**:
   Set the Content-Type header to application/json:
   ```
   Content-Type: application/json
   ```

3. **Request Body**:
   The body should be a JSON object containing:
   - `model`: Specifies which LLM to use (format: `<platform>:<model_name>`)
   - `messages`: An array of message objects representing the conversation
   - `stream`: A boolean indicating whether to stream the response (optional)

### Example cURL Request

```bash
curl -X POST \
  http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude:claude-3-5-sonnet-20240620",
    "messages": [{"role": "user", "content": "How does JVM work?"}],
    "stream": false
  }'
```

Here's how you construct value for the "model" field in the API request.
### Selecting Models

When making API requests, you can only use models from platforms that you've added to your `config.yaml` file. Use the format `<platform>:<model_name>` in the `model` field of your HTTP request.

Here's a table of supported platforms and their corresponding model names:


| Platform                | Supported Model names                                                                                                                                                                                                                                                                                                                                                                      |
|-------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| openai                  | gpt-3.5-turbo, gpt-3.5-turbo-1106, gpt-4o, gpt-4-turbo, gpt-4-turbo-preview, gpt-4-1106-preview, gpt-4-vision-preview, gpt-4, gpt-4-32k, text-embedding-3-large, text-embedding-3-small                                                                                                                                                                                                |
| gemini                  | gemini-1.5-pro-latest, gemini-1.0-pro-latest, gemini-1.0-pro-vision-latest, gemini-1.5-flash-latest, gemini-1.5-pro-latest, text-embedding-004                                                                                                                                                                                                                                                                |
| claude                  | claude-3-5-sonnet-20240620, claude-3-opus-20240229, claude-3-sonnet-20240229, claude-3-haiku-20240307                                                                                                                                                                                                                                                                                                              |
| mistral                 | open-mistral-7b, open-mixtral-8x7b, open-mixtral-8x22b, mistral-small-latest, mistral-large-latest, mistral-embed                                                                                                                                                                                                                                                                      |
| cohere                  | command-r, command-r-plus, embed-english-v3.0, embed-multilingual-v3.0                                                                                                                                                                                                                                                                                                                  |
| perplexity              | llama-3-sonar-small-32k-chat, llama-3-sonar-large-32k-chat, llama-3-8b-instruct, llama-3-70b-instruct, mixtral-8x7b-instruct                                                                                                                                                                                                                                                            |
| groq                    | llama3-8b-8192, llama3-70b-8192, mixtral-8x7b-32768, gemma-7b-it                                                                                                                                                                                                                                                                                                                        |
| vertexai                | gemini-1.0-pro-002, gemini-1.0-pro-vision-001, gemini-1.5-flash-preview-0514, gemini-1.5-pro-preview-0514, text-embedding-004, text-multilingual-embedding-002                                                                                                                                                                                                                         |
| vertexai-claude         | claude-3-opus@20240229, claude-3-sonnet@20240229, claude-3-haiku@20240307                                                                                                                                                                                                                                                                                                               |
| bedrock                 | anthropic.claude-3-opus-20240229-v1:0, anthropic.claude-3-sonnet-20240229-v1:0, anthropic.claude-3-haiku-20240307-v1:0, meta.llama3-8b-instruct-v1:0, meta.llama3-70b-instruct-v1:0, mistral.mistral-7b-instruct-v0:2, mistral.mixtral-8x7b-instruct-v0:1, mistral.mistral-large-2402-v1:0                                                                                                     |
| cloudflare              | @cf/meta/llama-3-8b-instruct, @cf/mistral/mistral-7b-instruct-v0.2-lora, @cf/google/gemma-7b-it-lora, @cf/qwen/qwen1.5-14b-chat-awq, @hf/thebloke/deepseek-coder-6.7b-instruct-awq                                                                                                                                                                                                     |
| replicate               | meta/meta-llama-3-70b-instruct, meta/meta-llama-3-8b-instruct, mistralai/mistral-7b-instruct-v0.2, mistralai/mixtral-8x7b-instruct-v0.1                                                                                                                                                                                                                                               |
| ernie                   | ernie-4.0-8k-preview, ernie-3.5-8k-preview, ernie-speed-128k, ernie-lite-8k, ernie-tiny-8k                                                                                                                                                                                                                                                                                             |
| qianwen                 | qwen-long, qwen-turbo, qwen-plus, qwen-max, qwen-max-longcontext, qwen-vl-plus, qwen-vl-max, text-embedding-v2                                                                                                                                                                                                                                                                         |
| moonshot                | moonshot-v1-8k, moonshot-v1-32k, moonshot-v1-128k                                                                                                                                                                                                                                                                                                                                      |
| deepseek                | deepseek-chat, deepseek-coder                                                                                                                                                                                                                                                                                                                                                           |
| zhipuai                 | glm-4, glm-4v, glm-3-turbo                                                                                                                                                                                                                                                                                                                                                              |
| anyscale                | meta-llama/Meta-Llama-3-8B-Instruct, meta-llama/Meta-Llama-3-70B-Instruct, codellama/CodeLlama-70b-Instruct-hf, mistralai/Mistral-7B-Instruct-v0.1, mistralai/Mixtral-8x7B-Instruct-v0.1, mistralai/Mixtral-8x22B-Instruct-v0.1, google/gemma-7b-it                                                                                                                                      |
| deepinfra               | meta-llama/Meta-Llama-3-8B-Instruct, meta-llama/Meta-Llama-3-70B-Instruct, mistralai/Mistral-7B-Instruct-v0.2, mistralai/Mixtral-8x7B-Instruct-v0.1, mistralai/Mixtral-8x22B-Instruct-v0.1, google/gemma-1.1-7b-it, databricks/dbrx-instruct, 01-ai/Yi-34B-Chat                                                                                                                       |
| fireworks               | accounts/fireworks/models/llama-v3-8b-instruct, accounts/fireworks/models/llama-v3-70b-instruct, accounts/fireworks/models/mistral-7b-instruct-v0p2, accounts/fireworks/models/mixtral-8x7b-instruct, accounts/fireworks/models/mixtral-8x22b-instruct, accounts/fireworks/models/qwen-72b-chat, accounts/fireworks/models/gemma-7b-it, accounts/fireworks/models/dbrx-instruct      |
| openrouter              | meta-llama/llama-3-8b-instruct, meta-llama/llama-3-8b-instruct:nitro, meta-llama/llama-3-8b-instruct:extended, meta-llama/llama-3-70b-instruct, meta-llama/llama-3-70b-instruct:nitro, mistralai/mistral-7b-instruct:free, codellama/codellama-70b-instruct, google/gemma-7b-it:free, 01-ai/yi-34b-chat, openai/gpt-3.5-turbo, openai/gpt-4o, openai/gpt-4-turbo, openai/gpt-4-turbo-preview, openai/gpt-4-vision-preview, openai/gpt-4, openai/gpt-4-32k, google/gemini-pro, google/gemini-pro-vision, google/gemini-pro-1.5, anthropic/claude-3-opus, anthropic/claude-3-sonnet, anthropic/claude-3-haiku, mistralai/mixtral-8x7b-instruct, mistralai/mixtral-8x22b-instruct, mistralai/mistral-small, mistralai/mistral-large, databricks/dbrx-instruct, cohere/command-r, cohere/command-r-plus  |
| octoai                  | meta-llama-3-8b-instruct, meta-llama-3-70b-instruct, mistral-7b-instruct, mixtral-8x7b-instruct, mixtral-8x22b-instruct                                                                                                                                                                                                                                                                 |
| together                | meta-llama/Llama-3-8b-chat-hf, meta-llama/Llama-3-70b-chat-hf, mistralai/Mistral-7B-Instruct-v0.2, mistralai/Mixtral-8x7B-Instruct-v0.1, mistralai/Mixtral-8x22B-Instruct-v0.1, google/gemma-7b-it, Qwen/Qwen1.5-72B-Chat, databricks/dbrx-instruct, zero-one-ai/Yi-34B-Chat, deepseek-ai/deepseek-llm-67b-chat, deepseek-ai/deepseek-coder-33b-instruct  |

## Examples of how to compose the model field in your API requests:

1. If you've configured `openai` in your config.yaml:
```json
"model": "openai:gpt-4-turbo"
```

2. If you've configured `gemini`:
```json
"model": "gemini:gemini-1.5-pro-latest"
```

3. If you've configured `claude`:
```json
"model": "claude:claude-3-opus-20240229"
```

4. If you've configured Mistral:
```json
"model": "mistral:mistral-large-latest"
```

5. If you've configured Cohere:
```json
"model": "cohere:command-r-plus"
```

## Multi-Platform Configuration and Usage

### Sample config.yaml

Here's a sample `config.yaml` that supports OpenAI, Gemini, and Claude:

```yaml
clients:
  - type: openai
    api_key: sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx  # Replace with your OpenAI API key
    api_base: https://api.openai.com/v1
    organization_id: org-xxxxxxxxxxxxxxxx  # Optional, replace if needed

  - type: gemini
    api_key: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx  # Replace with your Gemini API key
    patches:
      '.*':
        chat_completions_body:
          safetySettings:
            - category: HARM_CATEGORY_HARASSMENT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_HATE_SPEECH
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_SEXUALLY_EXPLICIT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_DANGEROUS_CONTENT
              threshold: BLOCK_NONE

  - type: claude
    api_key: sk-ant-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx  # Replace with your Claude API key
```

Remember to replace the placeholder API keys with your actual credentials.

### Sample curl Requests

Here are sample curl requests demonstrating how to use different models from each configured platform:

1. Using OpenAI's GPT-4 model:

```bash
curl -X POST \
  http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "openai:gpt-4o",
    "messages": [{"role": "user", "content": "Explain quantum computing in simple terms."}],
    "stream": false
  }'
```

2. Using Gemini's latest pro model:

```bash
curl -X POST \
  http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini:gemini-1.5-pro-latest",
    "messages": [{"role": "user", "content": "What are the main differences between machine learning and deep learning?"}],
    "stream": false
  }'
```

3. Using Claude's latest model:

```bash
curl -X POST \
  http://127.0.0.1:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude:claude-3-5-sonnet-20240620",
    "messages": [{"role": "user", "content": "Describe the process of photosynthesis."}],
    "stream": false
  }'
```

These examples demonstrate how you can easily switch between different AI models from various platforms by simply changing the `model` field in your API requests. The AI gateway handles the routing to the appropriate platform based on the model specified.

Remember that for each of these requests to work:
1. The corresponding platform (OpenAI, Gemini, or Claude) must be properly configured in your `config.yaml` file.
2. You must have valid API keys for each platform.
3. The model names used in the requests must match those supported by each platform (as listed in the model table provided earlier).

By setting up your `config.yaml` with multiple platforms and using the appropriate model names in your requests, you can leverage the power of various AI models through a single, unified API gateway.

## Roadmap

The journey of Agent Panel is just beginning. The roadmap includes several exciting features designed further to enhance the capability and efficiency of AI agents:

1. **Long-Term Centralized Memory:** Manage and optimize the memory usage across all agents, ensuring information retention and accessibility.
2. **Resuming Distributed Agent State:** This feature will allow the system to resume operations from the last state before a failure, preventing the need to restart processes from the beginning.
3. **Time Travel Debugging and Graph Visualization:** Developers can step through the agent calls to debug and visualize the decision-making process, enhancing understanding and troubleshooting capabilities.
4. **API Function Calling Decision-Making:** Rather than executing functions directly, the system will identify and suggest the best function to call based on the current context, optimizing performance and accuracy.
5. **Prompt Optimization Mode:** This will enable developers to iterate on prompts, compare responses, and refine their strategies to achieve the best outcomes.

## Thank you 
I want to thank @sigoden, author of Aichat, for his great work, which I could bank on building out the initial version.

## Get involved 
If any issues are running or using the project, please feel free to open an issue here. 
If you'd like to chat, please join the discord channel: https://discord.gg/DETRMEVqQ6 or stay tuned on X https://x.com/IncredibleDevHQ .  
If you like the project, remember to increment the star count!
