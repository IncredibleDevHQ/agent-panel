use super::{Client, ExtraConfig, Model, PromptType, QianwenClient, SendData, TokensCountFactors};

use crate::message;
use crate::{render::ReplyHandler, utils::PromptKind};
use crate::message::message::{Message, MessageRole};


use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::{
    Client as ReqwestClient, RequestBuilder,
};
use reqwest_eventsource::{Error as EventSourceError, Event, RequestBuilderExt};
use serde::Deserialize;
use serde_json::{json, Value};

const API_URL: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";

const MODELS: [(&str, usize, &str); 4] = [
    // https://help.aliyun.com/zh/dashscope/developer-reference/api-details
    ("qwen-max", 6000, "text"),
    ("qwen-max-longcontext", 28000, "text"),
    ("qwen-plus", 30000, "text"),
    ("qwen-turbo", 6000, "text"),
];

const TOKENS_COUNT_FACTORS: TokensCountFactors = (4, 14);

#[derive(Debug, Clone, Deserialize, Default)]
pub struct QianwenConfig {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub extra: Option<ExtraConfig>,
}

#[async_trait]
impl Client for QianwenClient {
    client_common_fns!();

    async fn send_message_inner(
        &self,
        client: &ReqwestClient,
        mut data: SendData,
    ) -> Result<Vec<Message>> {
        let api_key = self.get_api_key()?;
        let builder = self.request_builder(client, data)?;
        send_message(builder).await
    }

    async fn send_message_streaming_inner(
        &self,
        client: &ReqwestClient,
        handler: &mut ReplyHandler,
        mut data: SendData,
    ) -> Result<()> {
        let api_key = self.get_api_key()?;
        let builder = self.request_builder(client, data)?;
        send_message_streaming(builder, handler).await
    }
}

impl QianwenClient {
    config_get_fn!(api_key, get_api_key);

    pub const PROMPTS: [PromptType<'static>; 1] =
        [("api_key", "API Key:", true, PromptKind::String)];

    pub fn list_models(local_config: &QianwenConfig) -> Vec<Model> {
        let client_name = Self::name(local_config);
        MODELS
            .into_iter()
            .map(|(name, max_input_tokens, capabilities)| {
                Model::new(client_name, name)
                    .set_capabilities(capabilities.into())
                    .set_max_input_tokens(Some(max_input_tokens))
                    .set_tokens_count_factors(TOKENS_COUNT_FACTORS)
            })
            .collect()
    }

    fn request_builder(&self, client: &ReqwestClient, data: SendData) -> Result<RequestBuilder> {
        let api_key = self.get_api_key()?;

        let stream = data.stream;

        let url = API_URL;
        let (body, has_upload) = build_body(data, self.model.name.clone())?;

        log::debug!("Qianwen Request: {url} {body}");

        let mut builder = client.post(url).bearer_auth(api_key).json(&body);
        if stream {
            builder = builder.header("X-DashScope-SSE", "enable");
        }
        if has_upload {
            builder = builder.header("X-DashScope-OssResourceResolve", "enable");
        }

        Ok(builder)
    }
}

async fn send_message(builder: RequestBuilder) -> Result<Vec<Message>> {
    let data: Value = builder.send().await?.json().await?;
    check_error(&data)?;

    // Extract the "text" directly without checking for VL specific paths.
    let output = data["output"]["text"]
        .as_str()
        .ok_or_else(|| anyhow!("Unexpected response {data}"))?;
    // crate plain text message and return as array 
    let message = Message::PlainText {
        role: MessageRole::Assistant,
        content: output.to_string(),
    };

    Ok(vec![message])

}

async fn send_message_streaming(builder: RequestBuilder, handler: &mut ReplyHandler) -> Result<()> {
    let mut es = builder.eventsource()?;

    while let Some(event) = es.next().await {
        match event {
            Ok(Event::Open) => {}
            Ok(Event::Message(message)) => {
                let data: Value = serde_json::from_str(&message.data)?;
                check_error(&data)?;
                // Directly process the message data without checking for VL content.
                if let Some(text) = data["output"]["text"].as_str() {
                    handler.text(text)?;
                }
            }
            Err(err) => {
                match err {
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

fn check_error(data: &Value) -> Result<()> {
    if let (Some(code), Some(message)) = (data["code"].as_str(), data["message"].as_str()) {
        bail!("{code}: {message}");
    }
    Ok(())
}

fn build_body(data: SendData, model: String) -> Result<(Value, bool)> {
    let SendData {
        messages,
        functions,
        temperature,
        stream,
    } = data;

    let messages: Vec<Value> = messages
        .into_iter()
        .map(|message| match message {
            Message::FunctionReturn { role, content, .. }
            | Message::PlainText { role, content } => {
                json!({
                    "role": role,
                    "content": content,
                })
            },
            Message::FunctionCall { role, function_call, .. } => {
                // Construct a description for the function call including its arguments.
                let func_name = function_call.name.clone();
                let call_desc = format!("Function call: {} with arguments: {}", func_name, function_call.arguments);
                json!({
                    "role": role,
                    "content": call_desc,
                })
            },
        })
        .collect();

    let input = json!({ "messages": messages });

    let mut parameters = json!({});
    if let Some(v) = temperature {
        parameters["temperature"] = v.into();
    }
    if stream {
        parameters["incremental_output"] = true.into();
    }

    let mut body = json!({
        "model": model,
        "input": input,
        "parameters": parameters
    });

    if let Some(functions) = functions {
        body["functions"] = json!(functions);
    }

    let has_upload = false;

    Ok((body, has_upload))
}
