use crate::tiktoken::{cl100k_base, CoreBPE};
use fancy_regex::Regex;
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::env;
use std::sync::{Arc, Mutex};

lazy_static! {
    pub static ref CODE_BLOCK_RE: Regex = Regex::new(r"(?ms)```\w*(.*)```").unwrap();
}

/// Count how many tokens a piece of text needs to consume
pub fn count_tokens(text: &str) -> usize {
    cl100k_base_singleton()
        .lock()
        .expect("Failed to lock cl100k_base")
        .encode_with_special_tokens(text)
        .len()
}

pub fn cl100k_base_singleton() -> Arc<Mutex<CoreBPE>> {
    lazy_static! {
        static ref CL100K_BASE: Arc<Mutex<CoreBPE>> = Arc::new(Mutex::new(cl100k_base().unwrap()));
    }
    CL100K_BASE.clone()
}

pub fn detect_os() -> String {
    let os = env::consts::OS;
    if os == "linux" {
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if let Some(id) = line.strip_prefix("ID=") {
                    return format!("{os}/{id}");
                }
            }
        }
    }
    os.to_string()
}

pub fn sha256sum(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn detect_shell() -> (String, String, &'static str) {
    let os = env::consts::OS;
    if os == "windows" {
        if env::var("NU_VERSION").is_ok() {
            ("nushell".into(), "nu.exe".into(), "-c")
        } else if let Some(ret) = env::var("PSModulePath").ok().and_then(|v| {
            let v = v.to_lowercase();
            if v.split(';').count() >= 3 {
                if v.contains("powershell\\7\\") {
                    Some(("pwsh".into(), "pwsh.exe".into(), "-c"))
                } else {
                    Some(("powershell".into(), "powershell.exe".into(), "-Command"))
                }
            } else {
                None
            }
        }) {
            ret
        } else {
            ("cmd".into(), "cmd.exe".into(), "/C")
        }
    } else if env::var("NU_VERSION").is_ok() {
        ("nushell".into(), "nu".into(), "-c")
    } else {
        let shell_cmd = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell_name = match shell_cmd.rsplit_once('/') {
            Some((_, name)) => name.to_string(),
            None => shell_cmd.clone(),
        };
        let shell_name = if shell_name == "nu" {
            "nushell".into()
        } else {
            shell_name
        };
        (shell_name, shell_cmd, "-c")
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

pub type AbortSignal = Arc<AbortSignalInner>;

pub struct AbortSignalInner {
    ctrlc: AtomicBool,
    ctrld: AtomicBool,
}

pub fn create_abort_signal() -> AbortSignal {
    AbortSignalInner::new()
}

impl AbortSignalInner {
    pub fn new() -> AbortSignal {
        Arc::new(Self {
            ctrlc: AtomicBool::new(false),
            ctrld: AtomicBool::new(false),
        })
    }

    pub fn aborted(&self) -> bool {
        if self.aborted_ctrlc() {
            return true;
        }
        if self.aborted_ctrld() {
            return true;
        }
        false
    }

    pub fn aborted_ctrlc(&self) -> bool {
        self.ctrlc.load(Ordering::SeqCst)
    }

    pub fn aborted_ctrld(&self) -> bool {
        self.ctrld.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.ctrlc.store(false, Ordering::SeqCst);
        self.ctrld.store(false, Ordering::SeqCst);
    }

    pub fn set_ctrlc(&self) {
        self.ctrlc.store(true, Ordering::SeqCst);
    }

    pub fn set_ctrld(&self) {
        self.ctrld.store(true, Ordering::SeqCst);
    }
}

use inquire::{required, validator::Validation, Text};

const MSG_REQUIRED: &str = "This field is required";
const MSG_OPTIONAL: &str = "Optional field - Press ↵ to skip";

pub fn prompt_input_string(desc: &str, required: bool) -> anyhow::Result<String> {
    let mut text = Text::new(desc);
    if required {
        text = text.with_validator(required!(MSG_REQUIRED))
    } else {
        text = text.with_help_message(MSG_OPTIONAL)
    }
    let text = text.prompt()?;
    Ok(text)
}

pub fn prompt_input_integer(desc: &str, required: bool) -> anyhow::Result<String> {
    let mut text = Text::new(desc);
    if required {
        text = text.with_validator(|text: &str| {
            let out = if text.is_empty() {
                Validation::Invalid(MSG_REQUIRED.into())
            } else {
                validate_integer(text)
            };
            Ok(out)
        })
    } else {
        text = text
            .with_validator(|text: &str| {
                let out = if text.is_empty() {
                    Validation::Valid
                } else {
                    validate_integer(text)
                };
                Ok(out)
            })
            .with_help_message(MSG_OPTIONAL)
    }
    let text = text.prompt()?;
    Ok(text)
}

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    String,
    Integer,
}

pub fn init_tokio_runtime() -> anyhow::Result<tokio::runtime::Runtime> {
    use anyhow::Context;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .with_context(|| "Failed to init tokio")
}

fn validate_integer(text: &str) -> Validation {
    if text.parse::<i32>().is_err() {
        Validation::Invalid("Must be a integer".into())
    } else {
        Validation::Valid
    }
}

pub fn get_env_name(key: &str) -> String {
    format!(
        "{}_{}",
        env!("CARGO_CRATE_NAME").to_ascii_uppercase(),
        key.to_ascii_uppercase(),
    )
}

pub fn extract_block(input: &str) -> String {
    let output: String = CODE_BLOCK_RE
        .captures_iter(input)
        .filter_map(|m| {
            m.ok()
                .and_then(|cap| cap.get(1))
                .map(|m| String::from(m.as_str()))
        })
        .collect();
    if output.is_empty() {
        input.trim().to_string()
    } else {
        output.trim().to_string()
    }
}

pub fn now() -> String {
    let now = chrono::Local::now();
    now.to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}