# ADR-0004: Optional AI Provider Abstraction and Privacy Model

- **Status:** Accepted
- **Deciders:** Dhap Core Team / NH Prince Pradhan
- **Date:** 2026-10-22

---

## 1. Context and Problem Statement

Many modern developer tools lock core features behind mandatory AI subscriptions or cloud API keys. Furthermore, sending recorded screen frames or transcript data to remote cloud servers can violate company data policies, NDAs, or user privacy.

Dhap's core principle is: **"AI is optional, provider-independent, and human-in-the-loop."**

---

## 2. Decision: Trait-Based Provider Interface with Strict Privacy Gates

### 1. Zero Network Dependency by Default
- The entire application (recording, importing, frame extraction, step authoring, vector annotations, redaction, and Markdown/HTML/PDF exports) operates 100% offline.
- No API key or account is requested during onboarding.

### 2. The `AiProvider` Rust Trait
All AI interactions are mediated through an abstract, asynchronous trait in `crates/dhap-ai`:
```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn display_name(&self) -> &str;
    async fn generate_step_suggestions(
        &self,
        request: &StepSuggestionRequest,
    ) -> Result<Vec<StepSuggestion>, AiError>;
    async fn polish_instruction(
        &self,
        text: &str,
    ) -> Result<String, AiError>;
}
```

Implementations:
- `MockAiProvider`: Deterministic, offline provider for unit tests and CI testing without network calls or API keys.
- `OllamaProvider`: Local, private on-device LLM inference.
- Cloud adapters: Optional user-configured endpoints (OpenAI, Anthropic, Gemini).

### 3. Privacy and Consent Invariants
1. **Transmission Disclosure:** Before sending any data to a remote provider, the user is presented with a pre-flight modal detailing the exact payload (step text, transcript snippet, or redacted frame).
2. **Draft Status:** AI responses are inserted as `status: Draft` suggestions. The user must review, accept, or discard each suggestion.
3. **Key Protection:** Provider API keys are never stored inside `.dhap` project files, export folders, or git repositories. They are held in the operating system's secure credential store or runtime environment.
