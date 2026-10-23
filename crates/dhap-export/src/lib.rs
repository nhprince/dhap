use dhap_annotation::AnnotationRenderer;
use dhap_core::{ActionKind, DhapError, DhapResult, Project};
use dhap_media::AnimatedWalkthroughExporter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportTemplate {
    StandardGuide,
    SopCompliance,
    ClassroomHandout,
    GithubReadme,
}

impl FromStr for ExportTemplate {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "standard" | "standardguide" | "guide" => Ok(ExportTemplate::StandardGuide),
            "sop" | "sopcompliance" | "compliance" => Ok(ExportTemplate::SopCompliance),
            "classroom" | "classroomhandout" | "handout" | "lab" => {
                Ok(ExportTemplate::ClassroomHandout)
            }
            "github" | "githubreadme" | "readme" => Ok(ExportTemplate::GithubReadme),
            other => Err(format!(
                "Unknown template '{}'. Available: standard, sop, classroom, github",
                other
            )),
        }
    }
}

pub struct PreflightValidator;

impl PreflightValidator {
    /// Heuristic secret and credential scanner
    pub fn scan_secrets(text: &str) -> Vec<(&'static str, String)> {
        let mut detected = Vec::new();

        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
            if clean.starts_with("ghp_")
                || clean.starts_with("github_pat_")
                || clean.starts_with("gho_")
            {
                detected.push(("GitHub Access Token", clean.to_string()));
            } else if clean.starts_with("sk-")
                || clean.starts_with("sk_live_")
                || clean.starts_with("sk-ant-")
            {
                detected.push(("OpenAI/Anthropic API Key", clean.to_string()));
            } else if clean.starts_with("AKIA")
                && clean.len() == 20
                && clean.chars().all(|c| c.is_ascii_alphanumeric())
            {
                detected.push(("AWS Access Key ID", clean.to_string()));
            }
        }

        if text.contains("-----BEGIN") && text.contains("PRIVATE KEY-----") {
            detected.push((
                "Private Key Block",
                "-----BEGIN ... PRIVATE KEY-----".to_string(),
            ));
        }

        let lower = text.to_lowercase();
        if lower.contains("bearer ") && lower.contains("eyj") {
            detected.push(("JWT Bearer Token", "Bearer eyJ...".to_string()));
        }

        if (lower.contains("postgres://")
            || lower.contains("mongodb://")
            || lower.contains("mysql://")
            || lower.contains("redis://"))
            && lower.contains('@')
        {
            detected.push((
                "Database Connection URI with Credentials",
                "Database URI with credentials".to_string(),
            ));
        }

        detected
    }

    pub fn validate(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
    ) -> DhapResult<Vec<String>> {
        let mut warnings = Vec::new();

        if project.steps.is_empty() {
            return Err(DhapError::ExportPreflightFailed(
                "Project contains zero guide steps".to_string(),
            ));
        }

        for (idx, step) in project.steps.iter().enumerate() {
            if step.title.trim().is_empty() {
                return Err(DhapError::ExportPreflightFailed(format!(
                    "Step {} has an empty title",
                    idx + 1
                )));
            }

            // Check if referenced frames exist in raw_frames
            for fid in &step.frame_ids {
                if let Some(frame) = project.get_frame(fid) {
                    if !raw_frames.contains_key(&frame.relative_path) {
                        return Err(DhapError::ExportPreflightFailed(format!(
                            "Step '{}' references missing frame asset: {}",
                            step.title, frame.relative_path
                        )));
                    }
                }
            }

            // Comprehensive secret and credential scanner
            let combined = format!(
                "{} {} {} {}",
                step.title,
                step.instruction,
                step.expected_result.as_deref().unwrap_or(""),
                step.notes.as_deref().unwrap_or("")
            );

            for (secret_type, snippet) in Self::scan_secrets(&combined) {
                warnings.push(format!(
                    "Step {}: potential unredacted {} detected ('{}')",
                    idx + 1,
                    secret_type,
                    snippet
                ));
            }
        }

        Ok(warnings)
    }
}

pub struct MarkdownExporter;

impl MarkdownExporter {
    pub fn export(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
        output_dir: &Path,
    ) -> DhapResult<PathBuf> {
        Self::export_with_template(
            project,
            raw_frames,
            output_dir,
            ExportTemplate::StandardGuide,
        )
    }

    pub fn export_with_template(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
        output_dir: &Path,
        template: ExportTemplate,
    ) -> DhapResult<PathBuf> {
        PreflightValidator::validate(project, raw_frames)?;

        let assets_dir = output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        let mut md = String::new();

        match template {
            ExportTemplate::SopCompliance => {
                md.push_str(&format!(
                    "# Standard Operating Procedure: {}\n\n",
                    project.title
                ));
                md.push_str("| Document ID | Revision | Effective Date | Status |\n");
                md.push_str("|---|---|---|---|\n");
                md.push_str(&format!(
                    "| SOP-{} | v1.0 | {} | Approved / Active |\n\n",
                    &project.id[..8.min(project.id.len())],
                    project.created_at.format("%Y-%m-%d")
                ));
                md.push_str("## 1. Purpose & Objective\n\n");
                md.push_str(project.description.as_deref().unwrap_or(
                    "This standard operating procedure defines verified execution steps for reliable operations.",
                ));
                md.push_str("\n\n## 2. Operational Procedure\n\n");
            }
            ExportTemplate::ClassroomHandout => {
                md.push_str(&format!("# Lab Handout: {}\n\n", project.title));
                md.push_str("**Student Name:** ____________________  **Date:** ____________  **Score:** _____/100\n\n");
                md.push_str("> **Learning Objective:** Complete each hands-on step and verify your expected result.\n\n");
                md.push_str("## Lab Instructions & Checkpoints\n\n");
            }
            ExportTemplate::GithubReadme => {
                md.push_str(&format!("# {}\n\n", project.title));
                md.push_str("[![Generated with Dhap](https://img.shields.io/badge/guide-Dhap-orange.svg)](https://github.com/nhprince/dhap)\n\n");
                if let Some(desc) = &project.description {
                    md.push_str(&format!("{}\n\n", desc));
                }
                md.push_str("## Walkthrough Steps\n\n");
            }
            ExportTemplate::StandardGuide => {
                md.push_str(&format!("# {}\n\n", project.title));
                if let Some(desc) = &project.description {
                    md.push_str(&format!("{}\n\n", desc));
                }
                md.push_str("> *Generated with [Dhap (ধাপ)](https://github.com/nhprince/dhap) — Record once. Create the video and guide together.*\n\n");
                md.push_str("## Table of Contents\n\n");
                for (idx, step) in project.steps.iter().enumerate() {
                    let anchor = step.title.to_lowercase().replace(' ', "-");
                    md.push_str(&format!(
                        "{}. [{}]({}#step-{}-{})\n",
                        idx + 1,
                        step.title,
                        "",
                        idx + 1,
                        anchor
                    ));
                }
                md.push_str("\n---\n\n");
            }
        }

        for (idx, step) in project.steps.iter().enumerate() {
            let step_num = idx + 1;
            if template == ExportTemplate::ClassroomHandout {
                md.push_str(&format!("### Task {}: {}\n\n", step_num, step.title));
                md.push_str(&format!("- [ ] {}\n\n", step.instruction));
            } else {
                md.push_str(&format!("### Step {}: {}\n\n", step_num, step.title));
                md.push_str(&format!("{}\n\n", step.instruction));
            }

            for fid in &step.frame_ids {
                if let Some(frame) = project.get_frame(fid) {
                    if let Some(raw_bytes) = raw_frames.get(&frame.relative_path) {
                        let processed_bytes = AnnotationRenderer::process_image_bytes(
                            raw_bytes,
                            &frame.annotations,
                            &frame.redactions,
                        )?;

                        let asset_name = format!("step_{}_{}.png", step_num, frame.id);
                        let asset_path = assets_dir.join(&asset_name);
                        fs::write(&asset_path, processed_bytes)?;

                        md.push_str(&format!("![{}](assets/{})\n\n", step.title, asset_name));
                    }
                }
            }

            if let Some(expected) = &step.expected_result {
                if template == ExportTemplate::ClassroomHandout {
                    md.push_str(&format!(
                        "**Checkpoint Verification:** {}\n*Did your system match?* [ ] Yes  [ ] No\n\n",
                        expected
                    ));
                } else {
                    md.push_str(&format!("**Expected Result:** {}\n\n", expected));
                }
            }

            if let Some(notes) = &step.notes {
                md.push_str(&format!("*Note:* {}\n\n", notes));
            }

            md.push_str("---\n\n");
        }

        if template == ExportTemplate::SopCompliance {
            md.push_str("## 3. Verification & Sign-off\n\n");
            md.push_str("| Role | Printed Name | Signature | Date |\n");
            md.push_str("|---|---|---|---|\n");
            md.push_str("| Operator / Technician | | | |\n");
            md.push_str("| Quality Assurance / Peer | | | |\n\n");
        }

        let output_file = output_dir.join("README.md");
        fs::write(&output_file, md)?;

        Ok(output_file)
    }
}

pub struct HtmlExporter;

impl HtmlExporter {
    pub fn export(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
        output_dir: &Path,
    ) -> DhapResult<PathBuf> {
        Self::export_with_template(
            project,
            raw_frames,
            output_dir,
            ExportTemplate::StandardGuide,
        )
    }

    pub fn export_with_template(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
        output_dir: &Path,
        template: ExportTemplate,
    ) -> DhapResult<PathBuf> {
        PreflightValidator::validate(project, raw_frames)?;

        let assets_dir = output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        let template_badge_html = match template {
            ExportTemplate::SopCompliance => {
                format!(
                    r#"<div style="background: rgba(37,99,235,0.12); border: 1px solid #2563eb; border-radius: 8px; padding: 0.75rem 1rem; margin-bottom: 1.5rem; display: flex; justify-content: space-between; align-items: center;">
                        <div><strong style="color: #60a5fa; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 0.05em;">Standard Operating Procedure</strong><div style="font-size: 0.85rem; color: var(--text-muted);">SOP-{} • Rev 1.0</div></div>
                        <span style="background: #2563eb; color: #fff; font-size: 0.75rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: 4px;">COMPLIANCE VERIFIED</span>
                    </div>"#,
                    &project.id[..8.min(project.id.len())]
                )
            }
            ExportTemplate::ClassroomHandout => {
                r#"<div style="background: rgba(168,85,247,0.12); border: 1px solid #a855f7; border-radius: 8px; padding: 0.75rem 1rem; margin-bottom: 1.5rem; display: flex; justify-content: space-between; align-items: center;">
                    <div><strong style="color: #c084fc; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 0.05em;">Student Lab Handout</strong><div style="font-size: 0.85rem; color: var(--text-muted);">Hands-On Exercise • Checkpoints Required</div></div>
                    <span style="background: #a855f7; color: #fff; font-size: 0.75rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: 4px;">LAB WORKSHEET</span>
                </div>"#.to_string()
            }
            ExportTemplate::GithubReadme => {
                r#"<div style="background: rgba(249,115,22,0.12); border: 1px solid #f97316; border-radius: 8px; padding: 0.75rem 1rem; margin-bottom: 1.5rem; display: flex; justify-content: space-between; align-items: center;">
                    <div><strong style="color: #fb923c; text-transform: uppercase; font-size: 0.8rem; letter-spacing: 0.05em;">Open Source Walkthrough</strong><div style="font-size: 0.85rem; color: var(--text-muted);">Verified Reproducible Guide</div></div>
                    <span style="background: #f97316; color: #fff; font-size: 0.75rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: 4px;">DHAP EXPORT</span>
                </div>"#.to_string()
            }
            ExportTemplate::StandardGuide => String::new(),
        };

        let mut steps_html = String::new();

        for (idx, step) in project.steps.iter().enumerate() {
            let step_num = idx + 1;
            let mut images_html = String::new();

            for fid in &step.frame_ids {
                if let Some(frame) = project.get_frame(fid) {
                    if let Some(raw_bytes) = raw_frames.get(&frame.relative_path) {
                        let processed_bytes = AnnotationRenderer::process_image_bytes(
                            raw_bytes,
                            &frame.annotations,
                            &frame.redactions,
                        )?;

                        let asset_name = format!("step_{}_{}.png", step_num, frame.id);
                        let asset_path = assets_dir.join(&asset_name);
                        fs::write(&asset_path, processed_bytes)?;

                        images_html.push_str(&format!(
                            r#"<div class="frame-container"><img src="assets/{}" alt="{}" loading="lazy"/></div>"#,
                            asset_name, step.title
                        ));
                    }
                }
            }

            let action_badge_html = match &step.action_kind {
                Some(ActionKind::Click) => {
                    r#"<span class="action-pill action-click">👆 CLICK</span>"#
                }
                Some(ActionKind::Command) => {
                    r#"<span class="action-pill action-command">💻 COMMAND</span>"#
                }
                Some(ActionKind::Verify) => {
                    r#"<span class="action-pill action-verify">🔍 VERIFY</span>"#
                }
                Some(ActionKind::Type) => {
                    r#"<span class="action-pill action-type">⌨️ INPUT</span>"#
                }
                Some(ActionKind::Navigate) => {
                    r#"<span class="action-pill action-navigate">🌐 NAVIGATE</span>"#
                }
                Some(ActionKind::KeyShortcut) => {
                    r#"<span class="action-pill action-shortcut">⚡ SHORTCUT</span>"#
                }
                Some(ActionKind::Select) => {
                    r#"<span class="action-pill action-select">📋 SELECT</span>"#
                }
                Some(ActionKind::Wait) => r#"<span class="action-pill action-wait">⏳ WAIT</span>"#,
                Some(ActionKind::Custom(_c)) => "",
                None => "",
            };

            let code_html = match &step.code_snippet {
                Some(cs) => format!(
                    r#"<div class="code-container"><div class="code-header"><span>{}</span><button class="code-copy-btn" onclick="copySnippet(this, '{}')">Copy</button></div><pre><code>{}</code></pre></div>"#,
                    cs.language,
                    cs.code.replace('"', "&quot;").replace('\n', "\\n"),
                    cs.code.replace('<', "&lt;").replace('>', "&gt;")
                ),
                None => String::new(),
            };

            let expected_html = match &step.expected_result {
                Some(exp) => format!(
                    r#"<div class="callout expected"><strong>Expected Result:</strong> {}</div>"#,
                    exp
                ),
                None => String::new(),
            };

            let notes_html = match &step.notes {
                Some(n) => format!(
                    r#"<div class="callout note"><strong>Note:</strong> {}</div>"#,
                    n
                ),
                None => String::new(),
            };

            let tags_str = step.tags.join(" ");
            let search_corpus = format!(
                "{} {} {} {} {}",
                step.title,
                step.instruction,
                step.expected_result.as_deref().unwrap_or(""),
                step.notes.as_deref().unwrap_or(""),
                tags_str
            )
            .to_lowercase()
            .replace('"', "&quot;");

            let escaped_speech = format!("{}. {}", step.title, step.instruction)
                .replace('"', "&quot;")
                .replace('\'', "\\'")
                .replace('\n', " ");

            steps_html.push_str(&format!(
                r#"
                <article class="step-card" id="step-{}" data-search="{}">
                    <header class="step-header">
                        <div style="display:flex; align-items:center; gap:0.5rem; flex-wrap:wrap;">
                            <span class="step-badge">Step {}</span>
                            {}
                            <h2>{}</h2>
                        </div>
                    </header>
                    <div class="step-actions">
                        <button class="copy-btn" onclick="copyStepText(this, '{}')">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                            Copy Instruction
                        </button>
                        <button class="copy-btn" onclick="speakStep(this, '{}')">
                            🔊 Read Aloud
                        </button>
                        <label class="check-done">
                            <input type="checkbox" data-step="{}" onchange="toggleStepComplete(this, {})" />
                            Mark step completed
                        </label>
                    </div>
                    <p class="step-instruction">{}</p>
                    {}
                    {}
                    {}
                    {}
                </article>
                "#,
                step_num,
                search_corpus,
                step_num,
                action_badge_html,
                step.title,
                step.instruction.replace('"', "&quot;").replace('\n', " "),
                escaped_speech,
                step_num,
                step_num,
                step.instruction,
                code_html,
                images_html,
                expected_html,
                notes_html
            ));
        }

        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Dhap Guide</title>
    <style>
        :root {{
            --bg: #0b0f19;
            --surface: #131b2e;
            --surface-hover: #1e293b;
            --border: #23334d;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --primary: #f97316;
            --primary-hover: #ea580c;
            --accent-green: #22c55e;
            --radius: 12px;
            --font: system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, sans-serif;
        }}
        [data-theme="light"] {{
            --bg: #f8fafc;
            --surface: #ffffff;
            --surface-hover: #f1f5f9;
            --border: #e2e8f0;
            --text: #0f172a;
            --text-muted: #64748b;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--text);
            font-family: var(--font);
            line-height: 1.6;
            padding-bottom: 5rem;
            transition: background 0.2s ease, color 0.2s ease;
        }}
        /* Top Navigation Bar */
        .top-bar {{
            position: sticky;
            top: 0;
            background: rgba(11, 15, 25, 0.92);
            backdrop-filter: blur(12px);
            border-bottom: 1px solid var(--border);
            z-index: 100;
            padding: 0.65rem 1rem;
        }}
        [data-theme="light"] .top-bar {{
            background: rgba(248, 250, 252, 0.92);
        }}
        .top-bar-inner {{
            max-width: 900px;
            margin: 0 auto;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 0.75rem;
            flex-wrap: wrap;
        }}
        .search-wrap {{
            flex: 1;
            min-width: 180px;
        }}
        .search-input {{
            width: 100%;
            background: var(--surface);
            border: 1px solid var(--border);
            color: var(--text);
            padding: 0.35rem 0.75rem;
            border-radius: 8px;
            font-size: 0.85rem;
            outline: none;
        }}
        .search-input:focus {{
            border-color: var(--primary);
        }}
        .progress-info {{
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--text-muted);
            white-space: nowrap;
        }}
        .progress-bar-wrap {{
            flex: 1;
            min-width: 120px;
            height: 6px;
            background: var(--border);
            border-radius: 999px;
            overflow: hidden;
            margin: 0 0.5rem;
        }}
        .progress-bar-fill {{
            height: 100%;
            background: var(--primary);
            width: 0%;
            transition: width 0.25s ease;
        }}
        .nav-btn {{
            background: var(--surface);
            border: 1px solid var(--border);
            color: var(--text);
            padding: 0.35rem 0.7rem;
            border-radius: 8px;
            cursor: pointer;
            font-size: 0.82rem;
            font-weight: 600;
            display: inline-flex;
            align-items: center;
            gap: 0.35rem;
        }}
        .nav-btn:hover {{
            background: var(--surface-hover);
        }}
        .container {{
            max-width: 900px;
            margin: 2rem auto 0;
            padding: 0 1rem;
        }}
        header.guide-title-block {{
            margin-bottom: 2rem;
            text-align: center;
        }}
        header.guide-title-block h1 {{
            font-size: 2.25rem;
            font-weight: 800;
            margin-bottom: 0.5rem;
            letter-spacing: -0.025em;
        }}
        header.guide-title-block p {{
            color: var(--text-muted);
            font-size: 1.1rem;
        }}
        .celebration-banner {{
            background: linear-gradient(135deg, rgba(34, 197, 94, 0.2) 0%, rgba(16, 185, 129, 0.2) 100%);
            border: 2px solid var(--accent-green);
            color: var(--accent-green);
            padding: 1rem 1.25rem;
            border-radius: var(--radius);
            margin-bottom: 2rem;
            text-align: center;
            font-size: 1rem;
        }}
        .step-card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1.75rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
            transition: border-color 0.2s ease, opacity 0.2s ease;
        }}
        .step-card.completed {{
            border-color: var(--accent-green);
        }}
        .step-card.hidden {{
            display: none !important;
        }}
        .step-header {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 0.75rem;
        }}
        .step-badge {{
            background: var(--primary);
            color: white;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.85rem;
            font-weight: 700;
        }}
        .step-card.completed .step-badge {{
            background: var(--accent-green);
        }}
        .action-pill {{
            font-size: 0.72rem;
            font-weight: 800;
            letter-spacing: 0.05em;
            padding: 0.2rem 0.55rem;
            border-radius: 6px;
            text-transform: uppercase;
        }}
        .action-click {{ background: rgba(59, 130, 246, 0.2); color: #60a5fa; border: 1px solid rgba(59, 130, 246, 0.4); }}
        .action-command {{ background: rgba(168, 85, 247, 0.2); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.4); }}
        .action-verify {{ background: rgba(34, 197, 94, 0.2); color: #4ade80; border: 1px solid rgba(34, 197, 94, 0.4); }}
        .action-type {{ background: rgba(245, 158, 11, 0.2); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.4); }}
        .action-navigate {{ background: rgba(14, 165, 233, 0.2); color: #38bdf8; border: 1px solid rgba(14, 165, 233, 0.4); }}
        .action-shortcut {{ background: rgba(239, 68, 68, 0.2); color: #f87171; border: 1px solid rgba(239, 68, 68, 0.4); }}
        .action-select {{ background: rgba(99, 102, 241, 0.2); color: #818cf8; border: 1px solid rgba(99, 102, 241, 0.4); }}
        .action-wait {{ background: rgba(100, 116, 139, 0.2); color: #94a3b8; border: 1px solid rgba(100, 116, 139, 0.4); }}

        .code-container {{
            background: #0f172a;
            border: 1px solid var(--border);
            border-radius: 8px;
            margin: 1rem 0;
            overflow: hidden;
        }}
        .code-header {{
            background: #1e293b;
            padding: 0.35rem 0.75rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 0.75rem;
            color: var(--text-muted);
            text-transform: uppercase;
            font-weight: 700;
        }}
        .code-copy-btn {{
            background: none;
            border: 1px solid #334155;
            color: #cbd5e1;
            padding: 0.15rem 0.5rem;
            border-radius: 4px;
            cursor: pointer;
            font-size: 0.7rem;
        }}
        .code-copy-btn:hover {{ background: #334155; color: #fff; }}
        .code-container pre {{
            padding: 0.85rem 1rem;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 0.9rem;
            color: #38bdf8;
            overflow-x: auto;
        }}

        .step-header h2 {{
            font-size: 1.35rem;
            font-weight: 700;
        }}
        .step-actions {{
            display: flex;
            align-items: center;
            gap: 1rem;
            margin-bottom: 1rem;
            font-size: 0.85rem;
            color: var(--text-muted);
            flex-wrap: wrap;
        }}
        .copy-btn {{
            display: inline-flex;
            align-items: center;
            gap: 0.35rem;
            background: none;
            border: 1px solid var(--border);
            color: var(--text-muted);
            padding: 0.3rem 0.65rem;
            border-radius: 6px;
            cursor: pointer;
            font-size: 0.8rem;
        }}
        .copy-btn:hover {{
            background: var(--surface-hover);
            color: var(--text);
        }}
        .check-done {{
            display: inline-flex;
            align-items: center;
            gap: 0.4rem;
            cursor: pointer;
            user-select: none;
        }}
        .check-done input {{
            cursor: pointer;
            accent-color: var(--accent-green);
        }}
        .step-instruction {{
            font-size: 1.05rem;
            margin-bottom: 1.25rem;
        }}
        .frame-container {{
            margin: 1.25rem 0;
            border-radius: 8px;
            overflow: hidden;
            border: 1px solid var(--border);
            cursor: zoom-in;
            background: #000;
        }}
        .frame-container img {{
            width: 100%;
            height: auto;
            display: block;
            transition: transform 0.2s ease;
        }}
        .frame-container:hover img {{
            transform: scale(1.005);
        }}
        .callout {{
            padding: 0.875rem 1rem;
            border-radius: 8px;
            margin-top: 1rem;
            font-size: 0.95rem;
        }}
        .callout.expected {{
            background: rgba(34, 197, 94, 0.1);
            border-left: 4px solid var(--accent-green);
        }}
        .callout.note {{
            background: rgba(249, 115, 22, 0.1);
            border-left: 4px solid var(--primary);
        }}
        footer.guide-footer {{
            text-align: center;
            color: var(--text-muted);
            font-size: 0.9rem;
            margin-top: 3rem;
            padding-top: 1.5rem;
            border-top: 1px solid var(--border);
        }}
        /* Lightbox Modal */
        .lightbox {{
            position: fixed;
            inset: 0;
            background: rgba(0,0,0,0.85);
            backdrop-filter: blur(8px);
            display: none;
            align-items: center;
            justify-content: center;
            z-index: 200;
            cursor: zoom-out;
            padding: 1.5rem;
        }}
        .lightbox.active {{
            display: flex;
        }}
        .lightbox img {{
            max-width: 90vw;
            max-height: 90vh;
            border-radius: 8px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
        }}
        /* Slideshow Presentation Mode */
        body.slides-active .top-bar {{
            background: #000;
        }}
        body.slides-active .step-card {{
            display: none;
        }}
        body.slides-active .step-card.active-slide {{
            display: block !important;
            min-height: 60vh;
            border: 2px solid var(--primary);
        }}
        .slide-controls {{
            display: none;
            justify-content: center;
            align-items: center;
            gap: 1.5rem;
            margin: 2rem 0;
        }}
        body.slides-active .slide-controls {{
            display: flex;
        }}
        @media print {{
            .top-bar, .step-actions, .nav-btn, .search-wrap, .slide-controls {{ display: none !important; }}
            body {{ background: white !important; color: black !important; padding: 0 !important; }}
            .step-card {{ break-inside: avoid; border: 1px solid #ccc !important; box-shadow: none !important; margin-bottom: 2rem; }}
        }}
    </style>
</head>
<body>
    <div class="top-bar">
        <div class="top-bar-inner">
            <span class="progress-info" id="progressText">Progress: 0%</span>
            <div class="progress-bar-wrap">
                <div class="progress-bar-fill" id="progressBar"></div>
            </div>
            <div class="search-wrap">
                <input type="search" id="stepSearch" class="search-input" placeholder="🔍 Filter steps..." oninput="filterSteps(this.value)" />
            </div>
            <button class="nav-btn" onclick="togglePresentation()" id="slidesBtn">📺 Slides</button>
            <button class="nav-btn" onclick="toggleTheme()" id="themeBtn">Light Mode</button>
        </div>
    </div>

    <div class="container">
        <div id="celebrationBanner" class="celebration-banner" style="display: none;">
            🎉 <strong>All Steps Completed!</strong> Guide verified and saved to local progress.
        </div>

        {}
        <header class="guide-title-block">
            <h1>{}</h1>
            <p>{}</p>
        </header>

        <main id="stepsMain">
            {}
        </main>

        <div class="slide-controls">
            <button class="nav-btn" onclick="focusStep(currentStepFocus - 1)">◀ Previous</button>
            <span id="slideIndicator" style="font-weight: 700;">Slide 1 of {}</span>
            <button class="nav-btn" onclick="focusStep(currentStepFocus + 1)">Next ▶</button>
        </div>

        <footer class="guide-footer">
            <p>Made with <strong>Dhap (ধাপ)</strong> • Make the steps clear.</p>
        </footer>
    </div>

    <!-- Image Lightbox Modal -->
    <div class="lightbox" id="lightbox" onclick="closeLightbox()">
        <img id="lightboxImg" src="" alt="Enlarged screenshot" />
    </div>

    <script>
        const totalSteps = {};
        const storageKey = 'dhap_progress_' + '{}';
        let completedSteps = new Set();
        let isSlidesMode = false;
        let currentStepFocus = 1;

        // Restore saved progress
        try {{
            const saved = localStorage.getItem(storageKey);
            if (saved) {{
                const arr = JSON.parse(saved);
                arr.forEach(num => {{
                    completedSteps.add(num);
                    const card = document.getElementById('step-' + num);
                    if (card) {{
                        card.classList.add('completed');
                        const cb = card.querySelector('input[type="checkbox"]');
                        if (cb) cb.checked = true;
                    }}
                }});
            }}
        }} catch(e) {{}}

        updateProgress();

        function toggleStepComplete(checkbox, stepNum) {{
            const card = document.getElementById('step-' + stepNum);
            if (checkbox.checked) {{
                completedSteps.add(stepNum);
                card.classList.add('completed');
            }} else {{
                completedSteps.delete(stepNum);
                card.classList.remove('completed');
            }}
            try {{
                localStorage.setItem(storageKey, JSON.stringify(Array.from(completedSteps)));
            }} catch(e) {{}}
            updateProgress();
        }}

        function updateProgress() {{
            const pct = Math.round((completedSteps.size / totalSteps) * 100);
            document.getElementById('progressBar').style.width = pct + '%';
            document.getElementById('progressText').textContent = completedSteps.size + ' of ' + totalSteps + ' completed (' + pct + '%)';
            const banner = document.getElementById('celebrationBanner');
            if (completedSteps.size === totalSteps && totalSteps > 0) {{
                banner.style.display = 'block';
            }} else {{
                banner.style.display = 'none';
            }}
        }}

        function copyStepText(btn, text) {{
            navigator.clipboard.writeText(text).then(() => {{
                const originalText = btn.innerHTML;
                btn.innerHTML = '<span>✓ Copied!</span>';
                setTimeout(() => {{ btn.innerHTML = originalText; }}, 1800);
            }});
        }}

        function copySnippet(btn, code) {{
            navigator.clipboard.writeText(code).then(() => {{
                const orig = btn.textContent;
                btn.textContent = 'Copied!';
                setTimeout(() => {{ btn.textContent = orig; }}, 1600);
            }});
        }}

        function speakStep(btn, text) {{
            if (!('speechSynthesis' in window)) {{
                alert('Web Speech API is not supported in this browser.');
                return;
            }}
            window.speechSynthesis.cancel();
            const utterance = new SpeechSynthesisUtterance(text);
            utterance.rate = 1.0;
            btn.textContent = '🔊 Speaking...';
            utterance.onend = () => {{ btn.textContent = '🔊 Read Aloud'; }};
            utterance.onerror = () => {{ btn.textContent = '🔊 Read Aloud'; }};
            window.speechSynthesis.speak(utterance);
        }}

        function filterSteps(query) {{
            const q = query.trim().toLowerCase();
            document.querySelectorAll('.step-card').forEach(card => {{
                if (!q) {{
                    card.classList.remove('hidden');
                }} else {{
                    const corpus = card.getAttribute('data-search') || '';
                    if (corpus.includes(q)) {{
                        card.classList.remove('hidden');
                    }} else {{
                        card.classList.add('hidden');
                    }}
                }}
            }});
        }}

        function togglePresentation() {{
            isSlidesMode = !isSlidesMode;
            document.body.classList.toggle('slides-active', isSlidesMode);
            const btn = document.getElementById('slidesBtn');
            btn.textContent = isSlidesMode ? '📄 Full View' : '📺 Slides';
            if (isSlidesMode) {{
                focusStep(currentStepFocus);
            }} else {{
                document.querySelectorAll('.step-card').forEach(c => {{
                    c.classList.remove('active-slide');
                    c.style.outline = 'none';
                }});
            }}
        }}

        function focusStep(num) {{
            if (num < 1 || num > totalSteps) return;
            currentStepFocus = num;
            document.getElementById('slideIndicator').textContent = 'Slide ' + num + ' of ' + totalSteps;

            document.querySelectorAll('.step-card').forEach(c => {{
                c.classList.remove('active-slide');
                c.style.outline = 'none';
            }});

            const el = document.getElementById('step-' + num);
            if (el) {{
                if (isSlidesMode) {{
                    el.classList.add('active-slide');
                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                }} else {{
                    el.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                    el.style.outline = '2px solid var(--primary)';
                }}
            }}
        }}

        function toggleTheme() {{
            const current = document.body.getAttribute('data-theme');
            const target = current === 'light' ? 'dark' : 'light';
            document.body.setAttribute('data-theme', target);
            document.getElementById('themeBtn').textContent = target === 'light' ? 'Dark Mode' : 'Light Mode';
        }}

        // Setup Lightbox for screenshots
        document.querySelectorAll('.frame-container img').forEach(img => {{
            img.addEventListener('click', (e) => {{
                e.stopPropagation();
                document.getElementById('lightboxImg').src = img.src;
                document.getElementById('lightbox').classList.add('active');
            }});
        }});

        function closeLightbox() {{
            document.getElementById('lightbox').classList.remove('active');
        }}

        document.addEventListener('keydown', (e) => {{
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
            if (e.key === 'Escape') closeLightbox();
            else if (e.key === 'ArrowRight' || e.key === 'j') focusStep(currentStepFocus + 1);
            else if (e.key === 'ArrowLeft' || e.key === 'k') focusStep(currentStepFocus - 1);
            else if (e.key === 'p' || e.key === 'P') togglePresentation();
            else if (e.key === 'x') {{
                const el = document.getElementById('step-' + currentStepFocus);
                if (el) {{
                    const cb = el.querySelector('input[type="checkbox"]');
                    if (cb) {{
                        cb.checked = !cb.checked;
                        toggleStepComplete(cb, currentStepFocus);
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
            "#,
            project.title,
            template_badge_html,
            project.title,
            project
                .description
                .as_deref()
                .unwrap_or("A step-by-step walkthrough created with Dhap."),
            steps_html,
            project.steps.len(),
            project.steps.len(),
            project.id
        );

        let output_file = output_dir.join("index.html");
        fs::write(&output_file, full_html)?;

        Ok(output_file)
    }
}

pub struct JsonExporter;

impl JsonExporter {
    /// Exports the entire project, step metadata, and rich analytics as structured JSON
    pub fn export(project: &Project, output_dir: &Path) -> DhapResult<PathBuf> {
        fs::create_dir_all(output_dir)?;
        let stats = project.statistics();
        let payload = serde_json::json!({
            "project": project,
            "statistics": stats,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "schema_version": project.schema_version,
        });

        let output_file = output_dir.join("guide.json");
        fs::write(&output_file, serde_json::to_string_pretty(&payload)?)?;
        Ok(output_file)
    }
}

pub struct WalkthroughGifExporter;

impl WalkthroughGifExporter {
    /// Exports an animated demonstration GIF walkthrough from baked step frames
    pub fn export(
        project: &Project,
        raw_frames: &HashMap<String, Vec<u8>>,
        output_dir: &Path,
        delay_ms: u32,
    ) -> DhapResult<PathBuf> {
        PreflightValidator::validate(project, raw_frames)?;
        let assets_dir = output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        let mut processed_frames = Vec::new();
        for step in &project.steps {
            for fid in &step.frame_ids {
                if let Some(frame) = project.get_frame(fid) {
                    if let Some(raw_bytes) = raw_frames.get(&frame.relative_path) {
                        let baked = AnnotationRenderer::process_image_bytes(
                            raw_bytes,
                            &frame.annotations,
                            &frame.redactions,
                        )?;
                        processed_frames.push(baked);
                    }
                }
            }
        }

        if processed_frames.is_empty() {
            return Err(DhapError::ExportPreflightFailed(
                "Project contains no frames to export as GIF".to_string(),
            ));
        }

        let output_file = assets_dir.join("walkthrough.gif");
        AnimatedWalkthroughExporter::export_gif(&processed_frames, delay_ms, &output_file)?;
        Ok(output_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dhap_core::Frame;

    #[test]
    fn test_secret_scanner_detection() {
        let text = "Use export ghp_ABCDEF1234567890 securely and set sk-proj-1234567890abc";
        let detected = PreflightValidator::scan_secrets(text);
        assert_eq!(detected.len(), 2);
        assert_eq!(detected[0].0, "GitHub Access Token");
        assert_eq!(detected[1].0, "OpenAI/Anthropic API Key");

        let aws_text = "Deploying to AKIAIOSFODNN7EXAMPLE target";
        let detected_aws = PreflightValidator::scan_secrets(aws_text);
        assert_eq!(detected_aws.len(), 1);
        assert_eq!(detected_aws[0].0, "AWS Access Key ID");

        let priv_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIE...\n-----END RSA PRIVATE KEY-----";
        let detected_key = PreflightValidator::scan_secrets(priv_key);
        assert_eq!(detected_key.len(), 1);
        assert_eq!(detected_key[0].0, "Private Key Block");
    }

    #[test]
    fn test_export_templates_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let export_dir = temp_dir.path().join("templates");

        let mut project = Project::new("Compliance Procedure");
        let fid = "frame_sop".to_string();
        project.frames.push(Frame {
            id: fid.clone(),
            media_id: None,
            timestamp_ms: 1000,
            relative_path: "frames/sop.png".to_string(),
            width: 50,
            height: 50,
            annotations: Vec::new(),
            redactions: Vec::new(),
        });
        project.add_step(
            "Rotate Root Credentials",
            "Generate new keypairs and invalidate old session.",
            Some(fid),
            Some(1000),
        );

        let mut raw_frames = HashMap::new();
        let dummy_png = image::RgbaImage::new(10, 10);
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(dummy_png)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        raw_frames.insert("frames/sop.png".to_string(), buf);

        // SOP Markdown Export
        let sop_file = MarkdownExporter::export_with_template(
            &project,
            &raw_frames,
            &export_dir,
            ExportTemplate::SopCompliance,
        )
        .unwrap();
        let sop_content = fs::read_to_string(&sop_file).unwrap();
        assert!(sop_content.contains("Standard Operating Procedure"));
        assert!(sop_content.contains("Verification & Sign-off"));

        // Classroom Handout Markdown Export
        let lab_dir = temp_dir.path().join("lab");
        let lab_file = MarkdownExporter::export_with_template(
            &project,
            &raw_frames,
            &lab_dir,
            ExportTemplate::ClassroomHandout,
        )
        .unwrap();
        let lab_content = fs::read_to_string(&lab_file).unwrap();
        assert!(lab_content.contains("Lab Handout"));
        assert!(lab_content.contains("Student Name:"));
        assert!(lab_content.contains("- [ ] Generate new keypairs"));

        // HTML with template
        let html_file = HtmlExporter::export_with_template(
            &project,
            &raw_frames,
            &export_dir,
            ExportTemplate::SopCompliance,
        )
        .unwrap();
        let html_content = fs::read_to_string(&html_file).unwrap();
        assert!(html_content.contains("Standard Operating Procedure"));
        assert!(html_content.contains("COMPLIANCE VERIFIED"));
    }

    #[test]
    fn test_markdown_and_html_export_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let export_dir = temp_dir.path().join("out");

        let mut project = Project::new("Quickstart Tutorial");
        let fid = "frame_001".to_string();
        project.frames.push(Frame {
            id: fid.clone(),
            media_id: None,
            timestamp_ms: 1000,
            relative_path: "frames/step1.png".to_string(),
            width: 100,
            height: 100,
            annotations: Vec::new(),
            redactions: Vec::new(),
        });
        project.add_step(
            "Initialize Repo",
            "Run git init to start.",
            Some(fid),
            Some(1000),
        );

        let mut raw_frames = HashMap::new();
        // Generate a 10x10 dummy PNG
        let dummy_png = image::RgbaImage::new(10, 10);
        let mut buf = Vec::new();
        image::DynamicImage::ImageRgba8(dummy_png)
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        raw_frames.insert("frames/step1.png".to_string(), buf);

        // Export Markdown
        let md_file = MarkdownExporter::export(&project, &raw_frames, &export_dir).unwrap();
        assert!(md_file.exists());
        let md_content = fs::read_to_string(&md_file).unwrap();
        assert!(md_content.contains("# Quickstart Tutorial"));
        assert!(md_content.contains("Initialize Repo"));

        // Export HTML
        let html_file = HtmlExporter::export(&project, &raw_frames, &export_dir).unwrap();
        assert!(html_file.exists());
        let html_content = fs::read_to_string(&html_file).unwrap();
        assert!(html_content.contains("Quickstart Tutorial - Dhap Guide"));
        assert!(html_content.contains("Run git init to start."));

        // Export JSON
        let json_dir = temp_dir.path().join("json_out");
        let json_file = JsonExporter::export(&project, &json_dir).unwrap();
        assert!(json_file.exists());
        let json_content = fs::read_to_string(&json_file).unwrap();
        assert!(json_content.contains("Quickstart Tutorial"));
        assert!(json_content.contains("statistics"));

        // Export Walkthrough GIF
        let gif_dir = temp_dir.path().join("gif_out");
        let gif_file =
            WalkthroughGifExporter::export(&project, &raw_frames, &gif_dir, 500).unwrap();
        assert!(gif_file.exists());
        assert!(gif_file.extension().unwrap() == "gif");
    }
}
