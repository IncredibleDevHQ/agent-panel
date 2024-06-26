use super::*;

use anyhow::{bail, Context, Result};
use reqwest::{Client as ReqwestClient, RequestBuilder};
use serde::Deserialize;
use serde_json::{json, Value};

const API_BASE: &str = "https://api.anthropic.com/v1/messages";

#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeConfig {
    pub name: Option<String>,
    pub api_key: Option<String>,
    #[serde(default)]
    pub models: Vec<ModelData>,
    pub patches: Option<ModelPatches>,
    pub extra: Option<ExtraConfig>,
}

impl ClaudeClient {
    config_get_fn!(api_key, get_api_key);

    pub const PROMPTS: [PromptAction<'static>; 1] =
        [("api_key", "API Key:", true, PromptKind::String)];

    fn chat_completions_builder(
        &self,
        client: &ReqwestClient,
        data: ChatCompletionsData,
    ) -> Result<RequestBuilder> {
        let api_key = self.get_api_key().ok();

        let mut body = claude_build_chat_completions_body(data, &self.model)?;
        self.patch_chat_completions_body(&mut body);

        let url = API_BASE;

        debug!("Claude Request: {url} {body}");

        let mut builder = client.post(url).json(&body);
        builder = builder
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "tools-2024-05-16");
        if let Some(api_key) = api_key {
            builder = builder.header("x-api-key", api_key)
        }

        Ok(builder)
    }
}

impl_client_trait!(
    ClaudeClient,
    claude_chat_completions,
    claude_chat_completions_streaming
);

pub async fn claude_chat_completions(builder: RequestBuilder) -> Result<ChatCompletionsOutput> {
    let res = builder.send().await?;
    let status = res.status();
    let data: Value = res.json().await?;
    if !status.is_success() {
        catch_error(&data, status.as_u16())?;
    }
    debug!("non-stream-data: {data}");
    claude_extract_chat_completions(&data)
}

pub async fn claude_chat_completions_streaming(
    builder: RequestBuilder,
    handler: &mut SseHandler,
) -> Result<()> {
    let mut function_name = String::new();
    let mut function_arguments = String::new();
    let mut function_id = String::new();
    let handle = |message: SseMmessage| -> Result<bool> {
        let data: Value = serde_json::from_str(&message.data)?;
        debug!("stream-data: {data}");
        if let Some(typ) = data["type"].as_str() {
            match typ {
                "content_block_start" => {
                    if let (Some("tool_use"), Some(name), Some(id)) = (
                        data["content_block"]["type"].as_str(),
                        data["content_block"]["name"].as_str(),
                        data["content_block"]["id"].as_str(),
                    ) {
                        if !function_name.is_empty() {
                            let arguments: Value =
                                function_arguments.parse().with_context(|| {
                                    format!("Tool call '{function_name}' is invalid: arguments must be in valid JSON format")
                                })?;
                            handler.tool_call(ToolCall::new(
                                function_name.clone(),
                                arguments,
                                Some(function_id.clone()),
                            ))?;
                        }
                        function_name = name.into();
                        function_arguments.clear();
                        function_id = id.into();
                    }
                }
                "content_block_delta" => {
                    if let Some(text) = data["delta"]["text"].as_str() {
                        handler.text(text)?;
                    } else if let (true, Some(partial_json)) = (
                        !function_name.is_empty(),
                        data["delta"]["partial_json"].as_str(),
                    ) {
                        function_arguments.push_str(partial_json);
                    }
                }
                "content_block_stop" => {
                    if !function_name.is_empty() {
                        let arguments: Value = function_arguments.parse().with_context(|| {
                            format!("Tool call '{function_name}' is invalid: arguments must be in valid JSON format")
                        })?;
                        handler.tool_call(ToolCall::new(
                            function_name.clone(),
                            arguments,
                            Some(function_id.clone()),
                        ))?;
                    }
                }
                _ => {}
            }
        }
        Ok(false)
    };

    sse_stream(builder, handle).await
}

pub fn claude_build_chat_completions_body(
    data: ChatCompletionsData,
    model: &Model,
) -> Result<Value> {
    let ChatCompletionsData {
        mut messages,
        temperature,
        top_p,
        functions,
        stream,
    } = data;

    let system_message = extract_system_message(&mut messages);

    let mut network_image_urls = vec![];

    let messages: Vec<Value> = messages
        .into_iter()
        .flat_map(|message| {
            let Message { role, content } = message;
            match content {
                MessageContent::Text(text) => vec![json!({
                    "role": role,
                    "content": text,
                })],
                MessageContent::Array(list) => {
                    let content: Vec<_> = list
                        .into_iter()
                        .map(|item| match item {
                            MessageContentPart::Text { text } => {
                                json!({"type": "text", "text": text})
                            }
                            MessageContentPart::ImageUrl {
                                image_url: ImageUrl { url },
                            } => {
                                if let Some((mime_type, data)) = url
                                    .strip_prefix("data:")
                                    .and_then(|v| v.split_once(";base64,"))
                                {
                                    json!({
                                        "type": "image",
                                        "source": {
                                            "type": "base64",
                                            "media_type": mime_type,
                                            "data": data,
                                        }
                                    })
                                } else {
                                    network_image_urls.push(url.clone());
                                    json!({ "url": url })
                                }
                            }
                        })
                        .collect();
                    vec![json!({
                        "role": role,
                        "content": content,
                    })]
                }
                MessageContent::ToolResults((tool_call_results, text)) => {
                    let mut tool_call = vec![];
                    let mut tool_result = vec![];
                    if !text.is_empty() {
                        tool_call.push(json!({
                            "type": "text",
                            "text": text,
                        }))
                    }
                    for tool_call_result in tool_call_results {
                        tool_call.push(json!({
                            "type": "tool_use",
                            "id": tool_call_result.call.id,
                            "name": tool_call_result.call.name,
                            "input": tool_call_result.call.arguments,
                        }));
                        tool_result.push(json!({
                            "type": "tool_result",
                            "tool_use_id": tool_call_result.call.id,
                            "content": tool_call_result.output.to_string(),
                        }));
                    }
                    vec![
                        json!({
                            "role": "assistant",
                            "content": tool_call,
                        }),
                        json!({
                            "role": "user",
                            "content": tool_result,
                        }),
                    ]
                }
            }
        })
        .collect();

    if !network_image_urls.is_empty() {
        bail!(
            "The model does not support network images: {:?}",
            network_image_urls
        );
    }

    let mut body = json!({
        "model": model.name(),
        "messages": messages,
    });
    if let Some(v) = system_message {
        body["system"] = v.into();
    }
    if let Some(v) = model.max_tokens_param() {
        body["max_tokens"] = v.into();
    }
    if let Some(v) = temperature {
        body["temperature"] = v.into();
    }
    if let Some(v) = top_p {
        body["top_p"] = v.into();
    }
    if stream {
        body["stream"] = true.into();
    }
    if let Some(functions) = functions {
        body["tools"] = functions
            .iter()
            .map(|v| {
                json!({
                    "name": v.name,
                    "description": v.description,
                    "input_schema": v.parameters,
                })
            })
            .collect();
    }
    Ok(body)
}

pub fn claude_extract_chat_completions(data: &Value) -> Result<ChatCompletionsOutput> {
    let text = data["content"][0]["text"].as_str().unwrap_or_default();

    let mut tool_calls = vec![];
    if let Some(calls) = data["content"].as_array().map(|content| {
        content
            .iter()
            .filter(|content| matches!(content["type"].as_str(), Some("tool_use")))
            .collect::<Vec<&Value>>()
    }) {
        tool_calls = calls
            .into_iter()
            .filter_map(|call| {
                if let (Some(name), Some(input), Some(id)) = (
                    call["name"].as_str(),
                    call.get("input"),
                    call["id"].as_str(),
                ) {
                    Some(ToolCall::new(
                        name.to_string(),
                        input.clone(),
                        Some(id.to_string()),
                    ))
                } else {
                    None
                }
            })
            .collect();
    };

    if text.is_empty() && tool_calls.is_empty() {
        bail!("Invalid response data: {data}");
    }

    let output = ChatCompletionsOutput {
        text: text.to_string(),
        tool_calls,
        id: data["id"].as_str().map(|v| v.to_string()),
        input_tokens: data["usage"]["input_tokens"].as_u64(),
        output_tokens: data["usage"]["output_tokens"].as_u64(),
    };
    Ok(output)
}
