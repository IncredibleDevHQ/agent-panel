use anyhow::{Context, Result};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::client::{ensure_model_capabilities, init_client};
use crate::config::AIGatewayConfig;
use crate::config_files::ensure_parent_exists;
use crate::function_calling::{Function, FunctionCall};
use crate::input::Input;
use crate::utils::now;

use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct Message {
//     pub role: MessageRole,
//     pub content: String,
// }

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Message {
    FunctionReturn {
        id: Option<String>,
        role: MessageRole,
        name: String,
        content: String,
    },
    FunctionCall {
        id: Option<String>,
        role: MessageRole,
        function_call: FunctionCall,
        content: (),
    },
    // NB: This has to be the last variant as this enum is marked `#[serde(untagged)]`, so
    // deserialization will always try this variant last. Otherwise, it is possible to
    // accidentally deserialize a `FunctionReturn` value as `PlainText`.
    PlainText {
        role: MessageRole,
        content: String,
    },
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::FunctionReturn {
                id,
                role,
                name,
                content,
            } => {
                write!(
                    f,
                    "[{}] Function Return - ID: {:?}, Name: {}, Content: {}",
                    role, id, name, content
                )
            }
            Message::FunctionCall {
                id,
                role,
                function_call,
                content: _,
            } => {
                write!(
                    f,
                    "[{}] Function Call - ID: {:?}, {:?}",
                    role, id, function_call
                )
            }
            Message::PlainText { role, content } => {
                write!(f, "[{}] Plain Text: {}", role, content)
            }
        }
    }
}

impl Message {
    pub fn new_text(role: MessageRole, content: &str) -> Self {
        Self::PlainText {
            role: role.to_owned(),
            content: content.to_owned(),
        }
    }

    pub fn system(content: &str) -> Self {
        Self::new_text(MessageRole::System, content)
    }

    pub fn user(content: &str) -> Self {
        Self::new_text(MessageRole::User, content)
    }

    pub fn assistant(content: &str) -> Self {
        Self::new_text(MessageRole::Assistant, content)
    }

    pub fn function_call(id: Option<String>, call: &FunctionCall) -> Self {
        Self::FunctionCall {
            id,
            role: MessageRole::Assistant,
            function_call: call.clone(),
            content: (),
        }
    }

    pub fn function_return(id: Option<String>, name: &str, content: &str) -> Self {
        Self::FunctionReturn {
            id: id,
            role: MessageRole::Function.to_owned(),
            name: name.to_string(),
            content: content.to_string(),
        }
    }
}

impl From<&Message> for tiktoken_rs::ChatCompletionRequestMessage {
    fn from(m: &Message) -> tiktoken_rs::ChatCompletionRequestMessage {
        match m {
            Message::PlainText { role, content } => tiktoken_rs::ChatCompletionRequestMessage {
                role: role.to_string(),
                content: content.clone(),
                name: None,
            },
            Message::FunctionReturn {
                id,
                role,
                name,
                content,
            } => {
                let name_with_id = format!(
                    "{}{}",
                    name,
                    id.as_ref()
                        .map_or(String::new(), |id| format!(" (ID: {})", id))
                );
                tiktoken_rs::ChatCompletionRequestMessage {
                    role: role.to_string(),
                    content: content.clone(),
                    name: Some(name_with_id),
                }
            }
            Message::FunctionCall {
                id,
                role,
                function_call,
                content: _,
            } => {
                // Serialize the function_call to JSON, handle potential errors gracefully
                let function_call_json =
                    serde_json::to_string(&function_call).unwrap_or_else(|_| {
                        String::from("{\"error\":\"Failed to serialize function call\"}")
                    });

                let name_with_id = id
                    .as_ref()
                    .map_or(String::new(), |id| format!("Function call ID: {}", id));

                tiktoken_rs::ChatCompletionRequestMessage {
                    role: role.to_string(),
                    content: function_call_json,
                    name: Some(name_with_id),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    Assistant,
    User,
    Function,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Function => write!(f, "function"),
        }
    }
}

#[allow(dead_code)]
impl MessageRole {
    pub fn is_system(&self) -> bool {
        matches!(self, MessageRole::System)
    }

    pub fn is_user(&self) -> bool {
        matches!(self, MessageRole::User)
    }

    pub fn is_assistant(&self) -> bool {
        matches!(self, MessageRole::Assistant)
    }
    pub fn is_function(&self) -> bool {
        matches!(self, MessageRole::Function)
    }
}

impl AIGatewayConfig {
    pub async fn use_llm(
        &mut self,
        text: Option<String>,
        history: Option<Vec<Message>>,
        functions: Option<Vec<Function>>,
    ) -> Result<Vec<Message>> {
        let input = Input::new(text, functions, history);
        let mut client = init_client(self)?;
        ensure_model_capabilities(client.as_mut(), input.required_capabilities())?;

        let output = client.send_message(input.clone()).await?;
        log::debug!("Messages: {:#?}", output);
        Ok(output)
    }

    fn open_message_file(&self) -> Result<File> {
        let path = Self::messages_file()?;
        ensure_parent_exists(&path)?;
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to create/append {}", path.display()))
    }

    pub fn save_message(&mut self, input: Input, output: &str) -> Result<()> {
        self.last_message = Some((input.clone(), output.to_string()));

        let mut file = self.open_message_file()?;
        if output.is_empty() {
            return Ok(());
        }
        let timestamp = now();
        let summary = input.summary();
        let input_markdown = input.render();
        let output = format!(
            "# CHAT: {summary} [{timestamp}]\n{input_markdown}\n--------\n{output}\n--------\n\n",
        );
        file.write_all(output.as_bytes())
            .with_context(|| "Failed to save message")
    }
}
