use super::{openai::OpenAIConfig, ClientConfig, Model};
use crate::utils::{init_tokio_runtime, AbortSignal};
use std::{env, future::Future, time::Duration};
use tokio::time::sleep;

use crate::config::AIGatewayConfig;
use crate::function_calling::Function;
use crate::input::Input;
use crate::message::message::Message;
use crate::{
    render::ReplyHandler,
    utils::{prompt_input_integer, prompt_input_string, PromptKind},
};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, ClientBuilder, Proxy, RequestBuilder};
use serde::Deserialize;
use serde_json::{json, Value};

#[macro_export]
macro_rules! register_client {
    (
        $(($module:ident, $name:literal, $config:ident, $client:ident),)+
    ) => {
        $(
            mod $module;
        )+
        $(
            use self::$module::$config;
        )+

        #[derive(Debug, Clone, serde::Deserialize)]
        #[serde(tag = "type")]
        pub enum ClientConfig {
            $(
                #[serde(rename = $name)]
                $config($config),
            )+
            #[serde(other)]
            Unknown,
        }


        $(
            #[derive(Debug)]
            pub struct $client {
                global_config: $crate::config::AIGatewayConfig,
                config: $config,
                model: $crate::client::Model,
            }

            impl $client {
                pub const NAME: &'static str = $name;

                pub fn init(global_config: &$crate::config::AIGatewayConfig) -> Option<Box<dyn Client>> {
                    let model = global_config.model.clone();
                    log::debug!("model {:?}", model);
                    let config = global_config.clients.iter().find_map(|client_config| {
                        if let ClientConfig::$config(c) = client_config {
                            if Self::name(c) == &model.client_name {
                                return Some(c.clone())
                            }
                        }
                        None
                    })?;

                    Some(Box::new(Self {
                        global_config: global_config.clone(),
                        config,
                        model,
                    }))
                }

                pub fn name(config: &$config) -> &str {
                    config.name.as_deref().unwrap_or(Self::NAME)
                }
            }

        )+

        pub fn init_client(config: &$crate::config::AIGatewayConfig) -> anyhow::Result<Box<dyn Client>> {
            None
            $(.or_else(|| $client::init(config)))+
            .ok_or_else(|| {
                let model = config.model.clone();
                anyhow::anyhow!("Unknown client '{}'", &model.client_name)
            })
        }

        pub fn ensure_model_capabilities(client: &mut dyn Client, capabilities: $crate::client::ModelCapabilities) -> anyhow::Result<()> {
            if !client.model().capabilities.contains(capabilities) {
                let models = client.models();
                if let Some(model) = models.into_iter().find(|v| v.capabilities.contains(capabilities)) {
                    client.set_model(model);
                } else {
                    anyhow::bail!(
                        "The current model lacks the corresponding capability."
                    );
                }
            }
            Ok(())
        }

        pub fn list_client_types() -> Vec<&'static str> {
            vec![$($client::NAME,)+]
        }

        pub fn create_client_config(client: &str) -> anyhow::Result<(String, serde_json::Value)> {
            $(
                if client == $client::NAME {
                    return create_config(&$client::PROMPTS, $client::NAME)
                }
            )+
            anyhow::bail!("Unknown client {}", client)
        }

        pub fn list_models(config: &$crate::config::AIGatewayConfig) -> Vec<$crate::client::Model> {
            config
                .clients
                .iter()
                .flat_map(|v| match v {
                    $(ClientConfig::$config(c) => $client::list_models(c),)+
                    ClientConfig::Unknown => vec![],
                })
                .collect()
        }

    };
}

#[macro_export]
macro_rules! client_common_fns {
    () => {
        fn config(
            &self,
        ) -> (
            &$crate::config::AIGatewayConfig,
            &Option<$crate::client::ExtraConfig>,
        ) {
            (&self.global_config, &self.config.extra)
        }

        fn models(&self) -> Vec<Model> {
            Self::list_models(&self.config)
        }

        fn model(&self) -> &Model {
            &self.model
        }

        fn set_model(&mut self, model: Model) {
            self.model = model;
        }
    };
}

#[macro_export]
macro_rules! openai_compatible_client {
    ($client:ident) => {
        use crate::message::message::Message;
        #[async_trait]
        impl $crate::client::Client for $crate::client::$client {
            client_common_fns!();

            async fn send_message_inner(
                &self,
                client: &reqwest::Client,
                data: $crate::client::SendData,
            ) -> anyhow::Result<Vec<Message>> {
                let builder = self.request_builder(client, data)?;
                $crate::client::openai::openai_send_message(builder).await
            }

            async fn send_message_streaming_inner(
                &self,
                client: &reqwest::Client,
                handler: &mut $crate::render::ReplyHandler,
                data: $crate::client::SendData,
            ) -> Result<()> {
                let builder = self.request_builder(client, data)?;
                $crate::client::openai::openai_send_message_streaming(builder, handler).await
            }
        }
    };
}

#[macro_export]
macro_rules! config_get_fn {
    ($field_name:ident, $fn_name:ident) => {
        fn $fn_name(&self) -> anyhow::Result<String> {
            let api_key = self.config.$field_name.clone();
            api_key
                .or_else(|| {
                    let env_prefix = Self::name(&self.config);
                    let env_name =
                        format!("{}_{}", env_prefix, stringify!($field_name)).to_ascii_uppercase();
                    std::env::var(&env_name).ok()
                })
                .ok_or_else(|| anyhow::anyhow!("Miss {}", stringify!($field_name)))
        }
    };
}

#[async_trait]
pub trait Client: Send + Sync {
    fn config(&self) -> (&AIGatewayConfig, &Option<ExtraConfig>);

    fn models(&self) -> Vec<Model>;

    fn model(&self) -> &Model;

    fn set_model(&mut self, model: Model);

    fn build_client(&self) -> Result<ReqwestClient> {
        let mut builder = ReqwestClient::builder();
        let options = self.config().1;
        let timeout = options
            .as_ref()
            .and_then(|v| v.connect_timeout)
            .unwrap_or(10);
        let proxy = options.as_ref().and_then(|v| v.proxy.clone());
        builder = set_proxy(builder, &proxy)?;
        let client = builder
            .connect_timeout(Duration::from_secs(timeout))
            .build()
            .with_context(|| "Failed to build client")?;
        Ok(client)
    }

    async fn send_message(&self, input: Input) -> Result<Vec<Message>> {
        let config = self.config().0;
        // Ensure `build_client` and `prepare_send_data` do not block.
        let client = self.build_client().map_err(|e| {
            log::error!("Failed to build client: {:?}", e);
            e
        })?;

        // Similarly, handle errors during data preparation.
        let data = config.prepare_send_data(&input, false).map_err(|e| {
            log::error!("Failed to prepare send data: {:?}", e);
            e
        })?;

        // Directly await the async operation.
        match self.send_message_inner(&client, data).await {
            Ok(messages) => Ok(messages),
            Err(e) => {
                log::error!("Failed to send claude message: {:?}", e);
                Err(anyhow!("Failed to get answer: {}", e))
            }
        }
    }

    fn send_message_streaming(&self, input: &Input, handler: &mut ReplyHandler) -> Result<()> {
        async fn watch_abort(abort: AbortSignal) {
            loop {
                if abort.aborted() {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }
        }
        let abort = handler.get_abort();
        let input = input.clone();
        init_tokio_runtime()?.block_on(async move {
            tokio::select! {
                ret = async {
                    let global_config = self.config().0;
                    let client = self.build_client()?;
                    let data = global_config.prepare_send_data(&input, true)?;
                    self.send_message_streaming_inner(&client, handler, data).await
                } => {
                    handler.done()?;
                    ret.with_context(|| "Failed to get answer")
                }
                _ = watch_abort(abort.clone()) => {
                    handler.done()?;
                    Ok(())
                 },
            }
        })
    }

    async fn send_message_inner(
        &self,
        client: &ReqwestClient,
        data: SendData,
    ) -> Result<Vec<Message>>;

    async fn send_message_streaming_inner(
        &self,
        client: &ReqwestClient,
        handler: &mut ReplyHandler,
        data: SendData,
    ) -> Result<()>;
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::OpenAIConfig(OpenAIConfig::default())
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExtraConfig {
    pub proxy: Option<String>,
    pub connect_timeout: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SendData {
    pub messages: Vec<Message>,
    pub temperature: Option<f64>,
    pub stream: bool,
    // Optional field to represent the agent function calling data
    pub functions: Option<Vec<Function>>,
}

pub type PromptType<'a> = (&'a str, &'a str, bool, PromptKind);

pub fn create_config(list: &[PromptType], client: &str) -> Result<(String, Value)> {
    let mut config = json!({
        "type": client,
    });
    let mut model = client.to_string();
    for (path, desc, required, kind) in list {
        match kind {
            PromptKind::String => {
                let value = prompt_input_string(desc, *required)?;
                set_config_value(&mut config, path, kind, &value);
                if *path == "name" {
                    model = value;
                }
            }
            PromptKind::Integer => {
                let value = prompt_input_integer(desc, *required)?;
                set_config_value(&mut config, path, kind, &value);
            }
        }
    }

    let clients = json!(vec![config]);
    Ok((model, clients))
}

#[allow(unused)]
pub async fn send_message_as_streaming<F, Fut>(
    builder: RequestBuilder,
    handler: &mut ReplyHandler,
    f: F,
) -> Result<()>
where
    F: FnOnce(RequestBuilder) -> Fut,
    Fut: Future<Output = Result<String>>,
{
    let text = f(builder).await?;
    handler.text(&text)?;
    handler.done()?;

    Ok(())
}

// pub fn patch_system_message(messages: &mut Vec<Message>) {
//     if messages[0].role.is_system() {
//         let system_message = messages.remove(0);
//         if let (Some(message), MessageContent::Text(system_text)) =
//             (messages.get_mut(0), system_message.content)
//         {
//             if let MessageContent::Text(text) = message.content.clone() {
//                 message.content = MessageContent::Text(format!("{}\n\n{}", system_text, text))
//             }
//         }
//     }
// }

fn set_config_value(json: &mut Value, path: &str, kind: &PromptKind, value: &str) {
    let segs: Vec<&str> = path.split('.').collect();
    match segs.as_slice() {
        [name] => json[name] = to_json(kind, value),
        [scope, name] => match scope.split_once('[') {
            None => {
                if json.get(scope).is_none() {
                    let mut obj = json!({});
                    obj[name] = to_json(kind, value);
                    json[scope] = obj;
                } else {
                    json[scope][name] = to_json(kind, value);
                }
            }
            Some((scope, _)) => {
                if json.get(scope).is_none() {
                    let mut obj = json!({});
                    obj[name] = to_json(kind, value);
                    json[scope] = json!([obj]);
                } else {
                    json[scope][0][name] = to_json(kind, value);
                }
            }
        },
        _ => {}
    }
}

fn to_json(kind: &PromptKind, value: &str) -> Value {
    if value.is_empty() {
        return Value::Null;
    }
    match kind {
        PromptKind::String => value.into(),
        PromptKind::Integer => match value.parse::<i32>() {
            Ok(value) => value.into(),
            Err(_) => value.into(),
        },
    }
}

fn set_proxy(builder: ClientBuilder, proxy: &Option<String>) -> Result<ClientBuilder> {
    let proxy = if let Some(proxy) = proxy {
        if proxy.is_empty() || proxy == "false" || proxy == "-" {
            return Ok(builder);
        }
        proxy.clone()
    } else if let Ok(proxy) = env::var("HTTPS_PROXY").or_else(|_| env::var("ALL_PROXY")) {
        proxy
    } else {
        return Ok(builder);
    };
    let builder =
        builder.proxy(Proxy::all(&proxy).with_context(|| format!("Invalid proxy `{proxy}`"))?);
    Ok(builder)
}
