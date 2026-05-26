use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn default_transcription_api_url() -> String {
    "https://api.groq.com/openai/v1/audio/transcriptions".into()
}

fn default_transcription_model() -> String {
    "whisper-large-v3-turbo".into()
}

fn default_transcription_max_duration_secs() -> u64 {
    300
}

/// 语音转写配置（通过 Groq 提供 Whisper API）。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TranscriptionConfig {
    /// 是否为支持语音的 channel 启用语音转写。
    #[serde(default)]
    pub enabled: bool,
    /// Whisper API 端点 URL。
    #[serde(default = "default_transcription_api_url")]
    pub api_url: String,
    /// Whisper 模型名称。
    #[serde(default = "default_transcription_model")]
    pub model: String,
    /// 可选的语言提示，使用 ISO-639-1 编码，例如 `en`、`ru`。
    #[serde(default)]
    pub language: Option<String>,
    /// 允许转写的最大语音时长，单位为秒；超过该值的消息会被跳过。
    #[serde(default = "default_transcription_max_duration_secs")]
    pub max_duration_secs: u64,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_url: default_transcription_api_url(),
            model: default_transcription_model(),
            language: None,
            max_duration_secs: default_transcription_max_duration_secs(),
        }
    }
}
#[cfg(test)]
#[path = "transcription_tests.rs"]
mod transcription_tests;
