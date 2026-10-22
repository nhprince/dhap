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
}
