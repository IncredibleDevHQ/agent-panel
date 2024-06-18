use anyhow::{Context, Result};
use fancy_regex::Regex;
use indexmap::{IndexMap, IndexSet};
use inquire::{validator::Validation, Text};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    path::Path,
};
use threadpool::ThreadPool;

const BIN_DIR_NAME: &str = "bin";
const DECLARATIONS_FILE_PATH: &str = "functions.json";

lazy_static! {
    static ref THREAD_POOL: ThreadPool = ThreadPool::new(num_cpus::get());
}

pub type ToolResults = (Vec<ToolCallResult>, String);

pub fn need_send_call_results(arr: &[ToolCallResult]) -> bool {
    arr.iter().any(|v| !v.output.is_null())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCallResult {
    pub call: ToolCall,
    pub output: Value,
}

impl ToolCallResult {
    pub fn new(call: ToolCall, output: Value) -> Self {
        Self { call, output }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Function {
    names: IndexSet<String>,
    declarations: Vec<FunctionDeclaration>,
    #[cfg(windows)]
    bin_dir: std::path::PathBuf,
    env_path: Option<String>,
}

impl Function {
    pub fn init(functions_dir: &Path) -> Result<Self> {
        let bin_dir = functions_dir.join(BIN_DIR_NAME);
        let env_path = if bin_dir.exists() {
            prepend_env_path(&bin_dir).ok()
        } else {
            None
        };

        let declarations_file = functions_dir.join(DECLARATIONS_FILE_PATH);

        let declarations: Vec<FunctionDeclaration> = if declarations_file.exists() {
            let ctx = || {
                format!(
                    "Failed to load function declarations at {}",
                    declarations_file.display()
                )
            };
            let content = fs::read_to_string(&declarations_file).with_context(ctx)?;
            serde_json::from_str(&content).with_context(ctx)?
        } else {
            vec![]
        };

        let func_names = declarations.iter().map(|v| v.name.clone()).collect();

        Ok(Self {
            names: func_names,
            declarations,
            #[cfg(windows)]
            bin_dir,
            env_path,
        })
    }

    pub fn select(&self, matcher: &str) -> Option<Vec<FunctionDeclaration>> {
        let regex = Regex::new(&format!("^({matcher})$")).ok()?;
        let output: Vec<FunctionDeclaration> = self
            .declarations
            .iter()
            .filter(|v| regex.is_match(&v.name).unwrap_or_default())
            .cloned()
            .collect();
        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionConfig {
    pub enable: bool,
    pub declarations_file: String,
    pub functions_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: JsonSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    #[serde(rename = "type")]
    pub type_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<IndexMap<String, JsonSchema>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_value: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
    pub id: Option<String>,
}

impl ToolCall {
    pub fn dedup(calls: Vec<Self>) -> Vec<Self> {
        let mut new_calls = vec![];
        let mut seen_ids = HashSet::new();

        for call in calls.into_iter().rev() {
            if let Some(id) = &call.id {
                if !seen_ids.contains(id) {
                    seen_ids.insert(id.clone());
                    new_calls.push(call);
                }
            } else {
                new_calls.push(call);
            }
        }

        new_calls.reverse();
        new_calls
    }

    pub fn new(name: String, arguments: Value, id: Option<String>) -> Self {
        Self {
            name,
            arguments,
            id,
        }
    }
}

fn prepend_env_path(bin_dir: &Path) -> Result<String> {
    let current_path = std::env::var("PATH").context("No PATH environment variable")?;

    let new_path = if cfg!(target_os = "windows") {
        format!("{};{}", bin_dir.display(), current_path)
    } else {
        format!("{}:{}", bin_dir.display(), current_path)
    };
    Ok(new_path)
}

#[cfg(windows)]
fn polyfill_cmd_name(name: &str, bin_dir: &std::path::Path) -> String {
    let mut name = name.to_string();
    if let Ok(exts) = std::env::var("PATHEXT") {
        if let Some(cmd_path) = exts
            .split(';')
            .map(|ext| bin_dir.join(format!("{}{}", name, ext)))
            .find(|path| path.exists())
        {
            name = cmd_path.display().to_string();
        }
    }
    name
}
