#  agent-panel
Control panel and observability platform for optimizing and monitoring LLM/AI agents.


The current version only contains the AI gateway, which provides access 100+ LLMs across 20+ AI platforms effortlessly. The observability platform is work in progress.
The project is written in Rust.

## Supported LLMs

- OpenAI GPT-3.5/GPT-4 (paid, vision, embedding, function-calling)
- Gemini: Gemini-1.0/Gemini-1.5 (free, paid, vision, embedding, function-calling)
- Claude: Claude-3.5/Claude-3 (vision, paid, function-calling)
- Mistral (paid, embedding, function-calling)
- Cohere: Command-R/Command-R+ (paid, embedding, function-calling)
- Perplexity: Llama-3/Mixtral (paid)
- Groq: Llama-3/Mixtral/Gemma (free)
- Ollama (free, local, embedding)
- Azure OpenAI (paid, vision, embedding, function-calling)
- VertexAI: Gemini-1.0/Gemini-1.5 (paid, vision, embedding, function-calling)
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


## Configuration 
Rename the `example.config.yaml` to `config.yaml`. 
Once you decide which LLM's to use, here how you add the details in the config yaml file.

1. Decide which LLM platform and the model you want to use from the table below and add it to the model section, the first section of the configuration.

| Platform                | Supported Models                                                                                                                                                                                                                                                                                                                                                                      |
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

For instance if you wish to use Open AI's Gpt4-o, set the model field in the config.yaml to `openai:gpt-4o`. Use the <platform>:<model> format, ensure that the platform and the correspinding model is present in the table above.

The second necessary field is the `client` section of the config. 
Look for the the pkatforms section and fill in the fields. 

For instance, if you use open ai, here's the openai platform section in the clients.

```yaml
clients:
    - type: openai
    api_key: <your key>
    api_base: https://api.openai.com/v1               # ENV: {client}_API_BASE
    organization_id: <org id>
```

The gateway can support multiple models at the same time, so you can expand the clients section with multiple options. 
Let's say want to the Gateway to support both Claude and Gemini, you can expand as follows, 

Copy the -type: gemini and -type: claude sections from the example.config.yaml

```yaml
clients: 
      - type: openai
    api_key: <your key>
    api_base: https://api.openai.com/v1               # ENV: {client}_API_BASE
    organization_id: <org id>

     - type: gemini
    api_key: xxx                                      # ENV: {client}_API_KEY
    patches:
      '.*':                                           
        chat_completions_body:                        # Override safetySettings for all models
          safetySettings:
            - category: HARM_CATEGORY_HARASSMENT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_HATE_SPEECH
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_SEXUALLY_EXPLICIT
              threshold: BLOCK_NONE
            - category: HARM_CATEGORY_DANGEROUS_CONTENT
              threshold: BLOCK_NONE

  # See https://docs.anthropic.com/claude/reference/getting-started-with-the-api
     - type: claude
       api_key: sk-ant-xxx     
```     


