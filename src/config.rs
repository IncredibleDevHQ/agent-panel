use crate::client::{list_models, ClientConfig, Model, SendData};
use crate::input::Input;
use crate::message::message::{Message, MessageRole};
use crate::session::session::Session;
use crate::utils::get_env_name;
use std::{fs, path::Path};
use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::path::PathBuf;

const CLIENTS_FIELD: &str = "clients";

/// Monokai Extended
const DARK_THEME: &[u8] = include_bytes!("./assets/monokai-extended.theme.bin");
const LIGHT_THEME: &[u8] = include_bytes!("./assets/monokai-extended-light.theme.bin");

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AIGatewayConfig {
    #[serde(rename(serialize = "model", deserialize = "model"))]
    pub model_id: Option<String>,
    /// LLM temperature
    pub temperature: Option<f64>,
    /// Whether to save the session
    pub save_session: Option<bool>,
    /// Compress session if tokens exceed this value (>=1000)
    pub compress_threshold: usize,
    pub clients: Vec<ClientConfig>,
    #[serde(skip)]
    pub model: Model,
    #[serde(skip)]
    pub session: Option<Session>,
    #[serde(skip)]
    pub last_message: Option<(Input, String)>,
}

impl Default for AIGatewayConfig {
    fn default() -> Self {
        Self {
            model_id: None,
            temperature: None,
            save_session: None,
            compress_threshold: 2000,
            clients: vec![ClientConfig::default()],
            session: None,
            model: Default::default(),
            last_message: None,
        }
    }
}

impl AIGatewayConfig {
    pub fn new(yaml_path: &str) -> Result<AIGatewayConfig> {
        let file_path = Path::new(yaml_path);
    
        // Read the YAML content directly from the given path.
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read config file: {}", file_path.display()))?;
    
        // Deserialize the YAML string into AIGatewayConfig.
        let config = AIGatewayConfig::from_yaml(&content)?;
    
        Ok(config)
    }

   // create a new AI Gateway from the given YAML configuration content. 
    pub fn from_yaml(content: &str) -> Result<Self> {
        let mut config: AIGatewayConfig = serde_yaml::from_str(&content).map_err(|err| {
            let err_msg = err.to_string();
            if err_msg.starts_with(&format!("{}: ", CLIENTS_FIELD)) {
                anyhow!("clients: invalid value")
            } else {
                anyhow!("err_msg: {}", err_msg)
            }
        })?;

        config.setup_model()?;
        Ok(config)
    }

    fn setup_model(&mut self) -> Result<()> {
        let model = match &self.model_id {
            Some(v) => v.clone(),
            None => {
                log::debug!("Model id not set, using the first available model");
                let models = list_models(self);
                if models.is_empty() {
                    bail!("No available model");
                }

                models[0].id()
            }
        };
        self.set_model(&model)?;
        Ok(())
    }

    pub fn set_model(&mut self, value: &str) -> Result<()> {
        let models = list_models(self);
        let model = Model::find(&models, value);
        match model {
            None => bail!("Invalid model '{}'", value),
            Some(model) => {
                if let Some(session) = self.session.as_mut() {
                    session.set_model(model.clone())?;
                }
                self.model = model;
                Ok(())
            }
        }
    }

    pub fn config_dir() -> Result<PathBuf> {
        let env_name = get_env_name("config_dir");
        let path = if let Some(v) = env::var_os(env_name) {
            PathBuf::from(v)
        } else {
            let mut dir = dirs::config_dir().ok_or_else(|| anyhow!("Not found config dir"))?;
            dir.push(env!("CARGO_CRATE_NAME"));
            dir
        };
        Ok(path)
    }

    pub fn local_path(name: &str) -> Result<PathBuf> {
        let mut path = Self::config_dir()?;
        path.push(name);
        Ok(path)
    }

    pub fn build_messages(&self, input: &Input) -> Result<Vec<Message>> {
        let mut messages = vec![];
    
        // If both text and history are empty, return an error.
        if input.is_empty() && !input.history_exists() {
            bail!("Both text and history are empty.");
        }
    
        // If there's non-empty text, create a new user message from the input.
        if !input.is_empty() {
            let message_text = input.to_message();
            if !message_text.is_empty() {
                let message = Message::new_text(MessageRole::User, &message_text);
                messages.push(message);
            }
        }
    
        // If there's history, extend the messages with it.
        if let Some(history) = input.get_history() {
            messages.extend(history);
        }
        Ok(messages)
    }

    pub fn prepare_send_data(&self, input: &Input, stream: bool) -> Result<SendData> {
        let messages = self.build_messages(input)?;
        let temperature = self.temperature;

        self.model.max_input_tokens_limit(&messages)?;
        Ok(SendData {
            messages,
            temperature,
            stream,
            functions: input.function_calls(),
        })
    }
}
