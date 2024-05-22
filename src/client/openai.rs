use super::{ExtraConfig, Model, OpenAIClient, PromptType, SendData, TokensCountFactors};

use crate::{
    function_calling::FunctionCall, message::message::MessageRole, render::ReplyHandler,
    utils::PromptKind,
};

use anyhow::{bail, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{Client as ReqwestClient, RequestBuilder};
use reqwest_eventsource::{Error as EventSourceError, Event, RequestBuilderExt};
use serde::Deserialize;
use serde_json::{json, Value};

const API_BASE: &str = "https://api.openai.com/v1";

const MODELS: [(&str, usize, &str); 5] = [
    // https://platform.openai.com/docs/models/gpt-4-and-gpt-4-turbo
    ("gpt-4-turbo-preview", 128000, "text"),
    ("gpt-4-vision-preview", 128000, "text,vision"),
    ("gpt-4-1106-preview", 128000, "text"),
    // https://platform.openai.com/docs/models/gpt-3-5-turbo
    ("gpt-3.5-turbo", 16385, "text"),
    ("gpt-3.5-turbo-1106", 16385, "text"),
];

pub const OPENAI_TOKENS_COUNT_FACTORS: TokensCountFactors = (5, 2);

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenAIConfig {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub organization_id: Option<String>,
    pub extra: Option<ExtraConfig>,
}

openai_compatible_client!(OpenAIClient);

impl OpenAIClient {
    config_get_fn!(api_key, get_api_key);
    config_get_fn!(api_base, get_api_base);

    pub const PROMPTS: [PromptType<'static>; 1] =
        [("api_key", "API Key:", true, PromptKind::String)];

    pub fn list_models(local_config: &OpenAIConfig) -> Vec<Model> {
        let client_name = Self::name(local_config);
        MODELS
            .into_iter()
            .map(|(name, max_input_tokens, capabilities)| {
                Model::new(client_name, name)
                    .set_capabilities(capabilities.into())
                    .set_max_input_tokens(Some(max_input_tokens))
                    .set_tokens_count_factors(OPENAI_TOKENS_COUNT_FACTORS)
            })
            .collect()
    }

    fn request_builder(&self, client: &ReqwestClient, data: SendData) -> Result<RequestBuilder> {
        let api_key = self.get_api_key()?;
        let api_base = self.get_api_base().unwrap_or_else(|_| API_BASE.to_string());

        let body = openai_build_body(data, self.model.name.clone())?;

        let url = format!("{api_base}/chat/completions");

        log::debug!("OpenAI Request: {url} {body}");

        let mut builder = client.post(url).bearer_auth(api_key).json(&body);

        if let Some(organization_id) = &self.config.organization_id {
            builder = builder.header("OpenAI-Organization", organization_id);
        }

        Ok(builder)
    }
}

// we make the open ai call and parse the response 
// here is how a sample response looks like 

// {
//     "id": "chat_completion_123456",
//     "object": "chat.completion",
//     "created": 1616547890,
//     "model": "gpt-4-turbo",
//     "choices": [
//         {
//             "index": 0,
//             "logprobs": null,
//             "finish_reason": "length",
//             "text": "I checked the weather for you, and ",
//             "message": {
//                 "role": "assistant",
//                 "content": "It looks like it will be sunny in San Francisco today.",
//                 "tool_calls": [
//                     {
//                         "id": "tool_call_789",
//                         "type": "function",
//                         "function": {
//                             "name": "get_weather",
//                             "description": "Retrieves weather information for a specified location",
//                             "parameters": {
//                                 "location": "San Francisco, CA",
//                                 "date": "2024-04-10"
//                             }
//                         },
//                         "result": {
//                             "summary": "Sunny",
//                             "temperature": "68°F",
//                             "chance_of_rain": "10%"
//                         }
//                     }
//                 ]
//             }
//         }
//     ],
//     "usage": {
//         "prompt_tokens": 15,
//         "completion_tokens": 45,
//         "total_tokens": 60
//     }
// }

pub async fn openai_send_message(builder: RequestBuilder) -> Result<Vec<Message>> {
    let response = builder.send().await?;
    if !response.status().is_success() {
        let error_msg = response.text().await.unwrap_or_default();
        bail!("Request failed: {}", error_msg);
    }

    let data: Value = response.json().await?;
    if let Some(err_msg) = data["error"]["message"].as_str() {
        bail!("API error: {}", err_msg);
    }

    let messages = &data["choices"][0]["message"];
    let mut result_messages = Vec::new();

    if let Some(tool_calls) = messages["tool_calls"].as_array() {
        for tool_call in tool_calls {
            let id = tool_call["id"].as_str().map(String::from);
            let function = &tool_call["function"];
            let name = function["name"]
                .as_str()
                .map(String::from)
                .unwrap_or_default();
            let arguments = function["arguments"]
                .as_str()
                .unwrap_or_default()
                .to_string();

            result_messages.push(Message::FunctionCall {
                id,
                role: MessageRole::Assistant,
                function_call: FunctionCall { name, arguments },
                content: (),
            });
        }
    }

    if let Some(content) = messages["content"].as_str() {
        result_messages.push(Message::PlainText {
            role: MessageRole::Assistant,
            content: content.to_string(),
        });
    }

    if result_messages.is_empty() {
        bail!("No content or tool calls found in the response");
    }

    Ok(result_messages)
}

pub async fn openai_send_message_streaming(
    builder: RequestBuilder,
    handler: &mut ReplyHandler,
) -> Result<()> {
    let mut es = builder.eventsource()?;
    while let Some(event) = es.next().await {
        match event {
            Ok(Event::Open) => {}
            Ok(Event::Message(message)) => {
                if message.data == "[DONE]" {
                    break;
                }
                let data: Value = serde_json::from_str(&message.data)?;
                if let Some(text) = data["choices"][0]["delta"]["content"].as_str() {
                    handler.text(text)?;
                }
            }
            Err(err) => {
                match err {
                    EventSourceError::InvalidStatusCode(_, res) => {
                        let data: Value = res.json().await?;
                        if let Some(err_msg) = data["error"]["message"].as_str() {
                            bail!("{err_msg}");
                        } else if let Some(err_msg) = data["message"].as_str() {
                            bail!("{err_msg}");
                        } else {
                            bail!("Request failed, {data}");
                        }
                    }
                    EventSourceError::StreamEnded => {}
                    _ => {
                        bail!("{}", err);
                    }
                }
                es.close();
            }
        }
    }

    Ok(())
}

// Sample request containing the function call response, user and assistant message
// request also includes the function calling options provided.
// This move is to move away from the deprecated functions and use the use tool based function calling
// ref doc: https://platform.openai.com/docs/api-reference/chat/create (Look at function calling section)

// curl -X POST "https://api.example.com/v1/chat/completions" \
// -H "Content-Type: application/json" \
// -H "Authorization: Bearer $API_KEY" \
// -d '{
//   "model": "gpt-4-turbo",
//   "messages": [
//     {
//       "role": "user",
//       "content": "What\'s the weather like in Boston today?"
//     },
//     {
//       "role": "assistant",
//       "content": "Let me check that for you."
//     },
//     {
//       "role": "tool",
//       "content": [
//         {
//           "type": "function_call",
//           "id": "func123",
//           "name": "get_current_weather",
//           "input": {
//             "location": "Boston, MA",
//             "unit": "celsius"
//           }
//         }
//       ]
//     },
//     {
//       "role": "tool",
//       "content": [
//         {
//           "type": "tool_result",
//           "tool_use_id": "func123",
//           "content": "The current temperature in Boston is 15°C."
//         }
//       ]
//     }
//   ],
//   "tools": [
//     {
//       "type": "function",
//       "function": {
//         "name": "get_current_weather",
//         "description": "Get the current weather in a given location",
//         "parameters": {
//           "type": "object",
//           "properties": {
//             "location": {
//               "type": "string",
//               "description": "The city and state, e.g., Boston, MA"
//             },
//             "unit": {
//               "type": "string",
//               "enum": ["celsius", "fahrenheit"],
//               "description": "The temperature unit"
//             }
//           },
//           "required": ["location", "unit"]
//         }
//       }
//     }
//   ],
//   "tool_choice": "auto"
// }'

pub fn openai_build_body(data: SendData, model: String) -> Result<Value> {
    let messages: Vec<Value> = data
        .messages
        .into_iter()
        .map(|message| match message {
            Message::FunctionReturn {
                id,
                role,
                name,
                content,
            } => {
                json!({
                    "role": role,
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": id.unwrap_or_default(),
                        "content": content
                    }]
                })
            }
            Message::PlainText { role, content } => {
                json!({
                    "role": role,
                    "content": content
                })
            }
            _ => json!({}), // Intentionally return an empty JSON object for other cases
        })
        .filter(|msg| !msg.is_object() || (msg.as_object().map_or(false, |obj| !obj.is_empty())))
        .collect();

    let mut body = json!({
        "model": model,
        "messages": messages
    });

    // Serialize function calls according to the structure required by OpenAI
    if let Some(functions) = data.functions {
        let tools_json: Vec<Value> = functions
            .iter()
            .map(|func| {
                json!({
                    "type": "function",
                    "function": {
                        "name": func.name,
                        "description": func.description,
                        "parameters": {
                            "type": func.parameters._type,
                            "properties": func.parameters.properties,
                            "required": func.parameters.required
                        }
                    }
                })
            })
            .collect();
        body["tools"] = json!(tools_json);
    }

    if let Some(v) = data.temperature {
        body["temperature"] = json!(v);
    }
    if data.stream {
        body["stream"] = json!(true);
    }

    Ok(body)
}
