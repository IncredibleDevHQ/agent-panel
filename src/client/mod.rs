#[macro_use]
mod common;
mod model;

pub use common::*;
pub use model::*;

register_client!(
    (openai, "openai", OpenAIConfig, OpenAIClient),
    (gemini, "gemini", GeminiConfig, GeminiClient),
    (claude, "claude", ClaudeConfig, ClaudeClient),
    (mistral, "mistral", MistralConfig, MistralClient),
    (
        openai_compatible,
        "openai-compatible",
        OpenAICompatibleConfig,
        OpenAICompatibleClient
    ),
    (ollama, "ollama", OllamaConfig, OllamaClient),
    (
        azure_openai,
        "azure-openai",
        AzureOpenAIConfig,
        AzureOpenAIClient
    ),
    (vertexai, "vertexai", VertexAIConfig, VertexAIClient),
    (ernie, "ernie", ErnieConfig, ErnieClient),
    (qianwen, "qianwen", QianwenConfig, QianwenClient),
    (moonshot, "moonshot", MoonshotConfig, MoonshotClient),
);

pub const OPENAI_COMPATIBLE_PLATFORMS: [(&str, &str); 12] = [
    ("anyscale", "https://api.endpoints.anyscale.com/v1"),
    ("deepinfra", "https://api.deepinfra.com/v1/openai"),
    ("deepseek", "https://api.deepseek.com"),
    ("fireworks", "https://api.fireworks.ai/inference/v1"),
    ("groq", "https://api.groq.com/openai/v1"),
    ("mistral", "https://api.mistral.ai/v1"),
    ("moonshot", "https://api.moonshot.cn/v1"),
    ("openrouter", "https://openrouter.ai/api/v1"),
    ("octoai", "https://text.octoai.run/v1"),
    ("perplexity", "https://api.perplexity.ai"),
    ("together", "https://api.together.xyz/v1"),
    ("zhipuai", "https://open.bigmodel.cn/api/paas/v4"),
];