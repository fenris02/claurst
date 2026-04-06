// provider_id.rs — Branded newtypes for provider and model identifiers.
//
// ProviderId and ModelId are separate newtype wrappers around String so that
// the type system prevents accidentally passing a model name where a provider
// name is expected (and vice-versa).

use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ProviderId
// ---------------------------------------------------------------------------

/// A branded identifier for an LLM provider (e.g. "anthropic", "openai").
///
/// Well-known constants are provided as associated constants so callers do
/// not need to hard-code raw strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(String);

impl ProviderId {
    pub const AMAZON_BEDROCK: &'static str = "amazon-bedrock";
    // -----------------------------------------------------------------------
    // Well-known provider constants
    // -----------------------------------------------------------------------

    pub const ANTHROPIC: &'static str = "anthropic";
    pub const AZURE: &'static str = "azure";
    pub const BASETEN: &'static str = "baseten";
    pub const CEREBRAS: &'static str = "cerebras";
    pub const CLOUDFLARE: &'static str = "cloudflare";
    pub const CODEX: &'static str = "codex";
    pub const COHERE: &'static str = "cohere";
    pub const DEEPINFRA: &'static str = "deepinfra";
    pub const DEEPSEEK: &'static str = "deepseek";
    pub const FIREWORKS: &'static str = "fireworks";
    pub const FRIENDLI: &'static str = "friendli";
    pub const GITHUB_COPILOT: &'static str = "github-copilot";
    pub const GITLAB: &'static str = "gitlab";
    pub const GOOGLE: &'static str = "google";
    pub const GOOGLE_VERTEX: &'static str = "google-vertex";
    pub const GROQ: &'static str = "groq";
    pub const HUGGINGFACE: &'static str = "huggingface";
    pub const LLAMA_CPP: &'static str = "llama-cpp";
    pub const LM_STUDIO: &'static str = "lm-studio";
    pub const MISTRAL: &'static str = "mistral";
    pub const MOONSHOT: &'static str = "moonshotai";
    pub const NEBIUS: &'static str = "nebius";
    pub const NOVITA: &'static str = "novita";
    pub const NVIDIA: &'static str = "nvidia";
    pub const OLLAMA: &'static str = "ollama";
    pub const OPENAI: &'static str = "openai";
    pub const OPENROUTER: &'static str = "openrouter";
    pub const OVHCLOUD: &'static str = "ovhcloud";
    pub const PERPLEXITY: &'static str = "perplexity";
    pub const SAMBANOVA: &'static str = "sambanova";
    pub const SAP: &'static str = "sap";
    pub const SCALEWAY: &'static str = "scaleway";
    pub const SILICONFLOW: &'static str = "siliconflow";
    pub const STEPFUN: &'static str = "stepfun";
    pub const TOGETHER_AI: &'static str = "together-ai";
    pub const UPSTAGE: &'static str = "upstage";
    pub const VENICE: &'static str = "venice";
    pub const VULTR: &'static str = "vultr";
    pub const XAI: &'static str = "xai";
    pub const ZHIPU: &'static str = "zhipuai";

    /// Construct a `ProviderId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        ProviderId(s.into())
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for ProviderId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ProviderId {
    fn from(s: String) -> Self {
        ProviderId(s)
    }
}

impl From<&str> for ProviderId {
    fn from(s: &str) -> Self {
        ProviderId(s.to_string())
    }
}

impl PartialEq<str> for ProviderId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ProviderId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

// ---------------------------------------------------------------------------
// ModelId
// ---------------------------------------------------------------------------

/// A branded identifier for a model (e.g. "claude-opus-4-5", "gpt-4o").
///
/// Kept separate from `ProviderId` for type safety — you cannot accidentally
/// pass a model name where a provider name is expected.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(String);

impl ModelId {
    /// Construct a `ModelId` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        ModelId(s.into())
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for ModelId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for ModelId {
    fn from(s: String) -> Self {
        ModelId(s)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        ModelId(s.to_string())
    }
}

impl PartialEq<str> for ModelId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for ModelId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
