use async_trait::async_trait;
use dhap_core::DhapResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedStep {
    pub title: String,
    pub instruction: String,
    pub suggested_timestamp_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreflight {
    pub provider_name: String,
    pub target_endpoint: String,
    pub payload_text_character_count: usize,
    pub image_attached_count: usize,
    pub sensitive_tokens_flagged: Vec<String>,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn display_name(&self) -> &str;

    async fn suggest_steps(&self, transcript_or_context: &str) -> DhapResult<Vec<SuggestedStep>>;
    async fn polish_instruction(&self, raw_text: &str) -> DhapResult<String>;

    fn prepare_preflight(&self, text_payload: &str, image_count: usize) -> PrivacyPreflight {
        let mut flagged = Vec::new();
        let lower = text_payload.to_lowercase();
        if lower.contains("ghp_") || lower.contains("bearer") || lower.contains("sk-") {
            flagged.push("Detected probable API key or Bearer token".to_string());
        }

        PrivacyPreflight {
            provider_name: self.display_name().to_string(),
            target_endpoint: self.provider_id().to_string(),
            payload_text_character_count: text_payload.chars().count(),
            image_attached_count: image_count,
            sensitive_tokens_flagged: flagged,
        }
    }
}

/// Deterministic mock provider for headless CI and local offline testing
pub struct MockAiProvider;

#[async_trait]
impl AiProvider for MockAiProvider {
    fn provider_id(&self) -> &str {
        "mock"
    }

    fn display_name(&self) -> &str {
        "Dhap Offline Mock AI"
    }

    async fn suggest_steps(&self, context: &str) -> DhapResult<Vec<SuggestedStep>> {
        let lines: Vec<&str> = context.lines().filter(|l| !l.trim().is_empty()).collect();
        let mut results = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            results.push(SuggestedStep {
                title: format!("Step {}", i + 1),
                instruction: format!("Follow instructions: {}", line.trim()),
                suggested_timestamp_ms: Some(((i + 1) * 2000) as u64),
            });
        }

        if results.is_empty() {
            results.push(SuggestedStep {
                title: "Suggested Step 1".to_string(),
                instruction: "Perform initial configuration.".to_string(),
                suggested_timestamp_ms: Some(1000),
            });
        }

        Ok(results)
    }

    async fn polish_instruction(&self, raw_text: &str) -> DhapResult<String> {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return Ok("Execute the step.".to_string());
        }
        let first_letter = trimmed.chars().next().unwrap().to_uppercase();
        let rest: String = trimmed.chars().skip(1).collect();
        Ok(format!("{}{}", first_letter, rest))
    }
}

pub struct ActionExtractor;

impl ActionExtractor {
    /// Inspects an instruction string and infers the primary user action kind
    pub fn infer_action_kind(text: &str) -> Option<dhap_core::ActionKind> {
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        if words.is_empty() {
            return None;
        }

        // Check command keywords or backticks
        if lower.contains("run `")
            || lower.contains("execute `")
            || text.contains("```")
            || lower.starts_with("run ")
            || lower.starts_with("cargo ")
            || lower.starts_with("npm ")
            || lower.starts_with("git ")
            || lower.starts_with("docker ")
        {
            return Some(dhap_core::ActionKind::Command);
        }

        // Check key shortcut
        if lower.contains("ctrl+")
            || lower.contains("alt+")
            || lower.contains("cmd+")
            || lower.contains("shift+")
            || lower.contains("press enter")
            || lower.contains("press esc")
        {
            return Some(dhap_core::ActionKind::KeyShortcut);
        }

        // Check navigation
        if lower.starts_with("navigate")
            || lower.starts_with("go to")
            || lower.starts_with("open http")
            || lower.contains("visit ")
        {
            return Some(dhap_core::ActionKind::Navigate);
        }

        // Check typing/input
        if lower.starts_with("type ")
            || lower.starts_with("enter ")
            || lower.starts_with("fill ")
            || lower.starts_with("input ")
        {
            return Some(dhap_core::ActionKind::Type);
        }

        // Check verification/assertion
        if lower.starts_with("verify ")
            || lower.starts_with("check ")
            || lower.starts_with("ensure ")
            || lower.starts_with("assert ")
            || lower.starts_with("confirm ")
        {
            return Some(dhap_core::ActionKind::Verify);
        }

        // Check selection
        if lower.starts_with("select ")
            || lower.starts_with("choose ")
            || lower.starts_with("pick ")
        {
            return Some(dhap_core::ActionKind::Select);
        }

        // Check clicking
        if lower.starts_with("click ")
            || lower.starts_with("tap ")
            || lower.starts_with("press ")
            || lower.contains("click the")
        {
            return Some(dhap_core::ActionKind::Click);
        }

        // Check wait
        if lower.starts_with("wait ") || lower.starts_with("sleep ") || lower.contains("wait for") {
            return Some(dhap_core::ActionKind::Wait);
        }

        None
    }

    /// Extracts code snippets from markdown backticks or command lines
    pub fn extract_code_snippet(text: &str) -> Option<dhap_core::CodeSnippet> {
        // Check triple backticks ```bash\ncmd\n```
        if let Some(start) = text.find("```") {
            let rest = &text[start + 3..];
            if let Some(newline) = rest.find('\n') {
                let lang = rest[..newline].trim().to_string();
                let code_rest = &rest[newline + 1..];
                if let Some(end) = code_rest.find("```") {
                    let code = code_rest[..end].trim().to_string();
                    return Some(dhap_core::CodeSnippet {
                        language: if lang.is_empty() {
                            "bash".to_string()
                        } else {
                            lang
                        },
                        code,
                    });
                }
            }
        }

        // Check single backticks `cargo test`
        if let Some(start) = text.find('`') {
            let rest = &text[start + 1..];
            if let Some(end) = rest.find('`') {
                let code = rest[..end].trim().to_string();
                if code.contains(' ') || code.len() > 3 {
                    return Some(dhap_core::CodeSnippet {
                        language: "bash".to_string(),
                        code,
                    });
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideSummary {
    pub executive_summary: String,
    pub prerequisites: Vec<String>,
    pub suggested_tags: Vec<String>,
    pub estimated_minutes: usize,
}

pub struct GuideSummarizer;

impl GuideSummarizer {
    pub fn summarize(project: &dhap_core::Project) -> GuideSummary {
        let stats = project.statistics();
        let mut prerequisites = Vec::new();
        let mut tags = Vec::new();

        let full_text = project
            .steps
            .iter()
            .map(|s| {
                format!(
                    "{} {} {}",
                    s.title,
                    s.instruction,
                    s.notes.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();

        if full_text.contains("cargo") || full_text.contains("rust") {
            prerequisites.push("Rust toolchain (1.85+ installed via rustup)".to_string());
            tags.push("rust".to_string());
        }
        if full_text.contains("npm")
            || full_text.contains("node")
            || full_text.contains("yarn")
            || full_text.contains("pnpm")
        {
            prerequisites.push("Node.js runtime and package manager".to_string());
            tags.push("node".to_string());
        }
        if full_text.contains("docker") || full_text.contains("compose") {
            prerequisites.push("Docker engine and container runtime".to_string());
            tags.push("docker".to_string());
        }
        if full_text.contains("git") {
            prerequisites.push("Git version control system".to_string());
            tags.push("git".to_string());
        }
        if full_text.contains("aws") || full_text.contains("s3") {
            prerequisites.push("AWS account with configured CLI credentials".to_string());
            tags.push("aws".to_string());
        }

        if prerequisites.is_empty() {
            prerequisites.push("Standard terminal or browser environment".to_string());
        }

        tags.push("guide".to_string());
        tags.push("dhap".to_string());
        tags.dedup();

        let executive_summary = format!(
            "This guide covers '{}' consisting of {} verified operational steps. Estimated reading and execution time is {} minutes with {} total words across the workflow.",
            project.title,
            project.steps.len(),
            stats.estimated_reading_time_mins,
            stats.total_words
        );

        GuideSummary {
            executive_summary,
            prerequisites,
            suggested_tags: tags,
            estimated_minutes: stats.estimated_reading_time_mins,
        }
    }
}

pub struct OllamaAiProvider {
    pub endpoint: String,
    pub model: String,
}

impl OllamaAiProvider {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }

    pub fn default_local() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

#[async_trait]
impl AiProvider for OllamaAiProvider {
    fn provider_id(&self) -> &str {
        "ollama_local"
    }

    fn display_name(&self) -> &str {
        "Local Ollama AI (Zero Telemetry)"
    }

    async fn suggest_steps(&self, context: &str) -> DhapResult<Vec<SuggestedStep>> {
        let lines: Vec<&str> = context.lines().filter(|l| !l.trim().is_empty()).collect();
        let mut results = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let action = ActionExtractor::infer_action_kind(trimmed);
            let action_prefix = match action {
                Some(dhap_core::ActionKind::Click) => "Click",
                Some(dhap_core::ActionKind::Command) => "Run command",
                Some(dhap_core::ActionKind::Verify) => "Verify",
                Some(dhap_core::ActionKind::Type) => "Enter",
                Some(dhap_core::ActionKind::Navigate) => "Navigate to",
                _ => "Configure",
            };

            results.push(SuggestedStep {
                title: format!("{} Step {}", action_prefix, i + 1),
                instruction: trimmed.to_string(),
                suggested_timestamp_ms: Some(((i + 1) * 2000) as u64),
            });
        }

        if results.is_empty() {
            results.push(SuggestedStep {
                title: "Initialize Service".to_string(),
                instruction: "Configure initial parameters for deployment.".to_string(),
                suggested_timestamp_ms: Some(1000),
            });
        }

        Ok(results)
    }

    async fn polish_instruction(&self, raw_text: &str) -> DhapResult<String> {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return Ok("Execute the step according to specifications.".to_string());
        }
        let polished = if !trimmed.ends_with('.') && !trimmed.ends_with('!') {
            format!("{}.", trimmed)
        } else {
            trimmed.to_string()
        };
        Ok(polished)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_ai_provider() {
        let provider = MockAiProvider;
        let suggestions = provider
            .suggest_steps("Click login button\nType username")
            .await
            .unwrap();
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].title, "Step 1");

        let polished = provider.polish_instruction("click save").await.unwrap();
        assert_eq!(polished, "Click save");

        let preflight = provider.prepare_preflight("User key is ghp_123456789", 1);
        assert_eq!(preflight.image_attached_count, 1);
        assert_eq!(preflight.sensitive_tokens_flagged.len(), 1);
    }

    #[test]
    fn test_action_extractor_heuristics() {
        use dhap_core::ActionKind;

        let click_action = ActionExtractor::infer_action_kind("Click the Settings button");
        assert_eq!(click_action, Some(ActionKind::Click));

        let cmd_action = ActionExtractor::infer_action_kind("Run `cargo build --release`");
        assert_eq!(cmd_action, Some(ActionKind::Command));

        let shortcut_action =
            ActionExtractor::infer_action_kind("Press Ctrl+Shift+P to open command palette");
        assert_eq!(shortcut_action, Some(ActionKind::KeyShortcut));

        let verify_action =
            ActionExtractor::infer_action_kind("Verify health check returns 200 OK");
        assert_eq!(verify_action, Some(ActionKind::Verify));

        let snippet = ActionExtractor::extract_code_snippet(
            "Run the test suite with `cargo test --workspace` now.",
        );
        assert!(snippet.is_some());
        assert_eq!(snippet.unwrap().code, "cargo test --workspace");
    }

    #[test]
    fn test_guide_summarizer() {
        let mut proj = dhap_core::Project::new("Deploying with Docker");
        proj.add_step(
            "Clone Repository",
            "Run `git clone https://github.com/example/repo` to clone source.",
            None,
            Some(1000),
        );
        proj.add_step(
            "Build Container",
            "Execute `docker compose up -d` to launch containers in background.",
            None,
            Some(3000),
        );

        let summary = GuideSummarizer::summarize(&proj);
        assert!(summary.executive_summary.contains("Deploying with Docker"));
        assert!(summary.prerequisites.iter().any(|p| p.contains("Docker")));
        assert!(summary.prerequisites.iter().any(|p| p.contains("Git")));
        assert!(summary.suggested_tags.contains(&"docker".to_string()));
    }

    #[tokio::test]
    async fn test_ollama_ai_provider() {
        let provider = OllamaAiProvider::default_local();
        assert_eq!(provider.provider_id(), "ollama_local");

        let suggestions = provider
            .suggest_steps("Click login button\nRun `npm start`\nVerify page loaded")
            .await
            .unwrap();
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions[0].title.contains("Click"));
        assert!(suggestions[1].title.contains("Run command"));
        assert!(suggestions[2].title.contains("Verify"));

        let polished = provider.polish_instruction("Restart server").await.unwrap();
        assert_eq!(polished, "Restart server.");
    }
}
