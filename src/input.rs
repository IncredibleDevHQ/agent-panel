use crate::client::ModelCapabilities;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::message::message::Message;
use super::function_calling::Function;

lazy_static! {
    static ref URL_RE: Regex = Regex::new(r"^[A-Za-z0-9_-]{2,}:/").unwrap();
}

#[derive(Debug, Clone)]
pub struct Input {
    text: Option<String>,
    functions: Option<Vec<Function>>,
    history: Option<Vec<Message>>,
}

impl Input {
    pub fn from_str(text: &str) -> Self {
        Self {
            text: Some(text.to_string()),
            functions: Default::default(),
            history: Default::default(),
        }
    }

    pub fn new(
        text: Option<String>,
        functions: Option<Vec<Function>>,
        history: Option<Vec<Message>>,
    ) -> Self {
        Self {
            text: text,
            functions,
            history,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_none()
    }

    pub fn history_exists(&self) -> bool {
        self.history.is_some()
    }

    pub fn get_history(&self) -> Option<Vec<Message>> {
        self.history.clone()
    }

    pub fn function_calls_exists(&self) -> bool {
        self.functions.is_some()
    }

    pub fn function_calls(&self) -> Option<Vec<Function>> {
        self.functions.clone()
    }

    pub fn summary(&self) -> String {
        if self.text.is_none() {
            return "".to_string();
        }
        let text: String = self
            .text
            .clone()
            .unwrap()
            .trim()
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect();
        if text.width_cjk() > 70 {
            let mut sum_width = 0;
            let mut chars = vec![];
            for c in text.chars() {
                sum_width += c.width_cjk().unwrap_or(1);
                if sum_width > 67 {
                    chars.extend(['.', '.', '.']);
                    break;
                }
                chars.push(c);
            }
            chars.into_iter().collect()
        } else {
            text
        }
    }

    pub fn render(&self) -> String {
        if self.text.is_none() {
            return "".to_string();
        }
        self.text.as_ref().unwrap().clone()
    }

    pub fn to_message(&self) -> String {
        if self.text.is_none() {
            return "".to_string();
        }
        self.text.clone().unwrap()
    }

    // Without media, we assume only text capabilities are needed.
    pub fn required_capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::Text
    }
}
