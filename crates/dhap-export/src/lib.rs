use dhap_annotation::AnnotationRenderer;
use dhap_core::{DhapError, DhapResult, Project};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PreflightValidator;

impl PreflightValidator {
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

            // Heuristic privacy scanner in step text
            let text_lower = format!("{} {}", step.title, step.instruction).to_lowercase();
            if text_lower.contains("ghp_")
                || text_lower.contains("bearer ")
                || text_lower.contains("api_key")
                || text_lower.contains("sk-")
            {
                warnings.push(format!(
                    "Step {}: potential unredacted credential or API key in text instructions",
                    idx + 1
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
        PreflightValidator::validate(project, raw_frames)?;

        let assets_dir = output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        let mut md = String::new();
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

        for (idx, step) in project.steps.iter().enumerate() {
            let step_num = idx + 1;
            md.push_str(&format!("### Step {}: {}\n\n", step_num, step.title));
            md.push_str(&format!("{}\n\n", step.instruction));

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
                md.push_str(&format!("**Expected Result:** {}\n\n", expected));
            }

            if let Some(notes) = &step.notes {
                md.push_str(&format!("*Note:* {}\n\n", notes));
            }

            md.push_str("---\n\n");
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
        PreflightValidator::validate(project, raw_frames)?;

        let assets_dir = output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

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

            steps_html.push_str(&format!(
                r#"
                <article class="step-card" id="step-{}">
                    <header class="step-header">
                        <span class="step-badge">Step {}</span>
                        <h2>{}</h2>
                    </header>
                    <p class="step-instruction">{}</p>
                    {}
                    {}
                    {}
                </article>
                "#,
                step_num,
                step_num,
                step.title,
                step.instruction,
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
            --bg: #0f172a;
            --surface: #1e293b;
            --border: #334155;
            --text: #f8fafc;
            --text-muted: #94a3b8;
            --primary: #f97316;
            --primary-hover: #ea580c;
            --radius: 12px;
            --font: system-ui, -apple-system, 'Segoe UI', Roboto, Helvetica, sans-serif;
        }}
        @media (prefers-color-scheme: light) {{
            :root {{
                --bg: #f8fafc;
                --surface: #ffffff;
                --border: #e2e8f0;
                --text: #0f172a;
                --text-muted: #64748b;
            }}
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--text);
            font-family: var(--font);
            line-height: 1.6;
            padding: 2rem 1rem;
        }}
        .container {{
            max-width: 860px;
            margin: 0 auto;
        }}
        header.guide-title-block {{
            margin-bottom: 2.5rem;
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
        .step-card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: var(--radius);
            padding: 1.75rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1);
        }}
        .step-header {{
            display: flex;
            align-items: center;
            gap: 0.75rem;
            margin-bottom: 1rem;
        }}
        .step-badge {{
            background: var(--primary);
            color: white;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.85rem;
            font-weight: 700;
        }}
        .step-header h2 {{
            font-size: 1.35rem;
            font-weight: 700;
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
        }}
        .frame-container img {{
            width: 100%;
            height: auto;
            display: block;
        }}
        .callout {{
            padding: 0.875rem 1rem;
            border-radius: 8px;
            margin-top: 1rem;
            font-size: 0.95rem;
        }}
        .callout.expected {{
            background: rgba(34, 197, 94, 0.1);
            border-left: 4px solid #22c55e;
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
        @media print {{
            body {{ background: white; color: black; }}
            .step-card {{ break-inside: avoid; border: 1px solid #ccc; box-shadow: none; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header class="guide-title-block">
            <h1>{}</h1>
            <p>{}</p>
        </header>

        <main>
            {}
        </main>

        <footer class="guide-footer">
            <p>Made with <strong>Dhap (ধাপ)</strong> • Make the steps clear.</p>
        </footer>
    </div>
</body>
</html>
            "#,
            project.title,
            project.title,
            project
                .description
                .as_deref()
                .unwrap_or("A step-by-step walkthrough created with Dhap."),
            steps_html
        );

        let output_file = output_dir.join("index.html");
        fs::write(&output_file, full_html)?;

        Ok(output_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dhap_core::Frame;

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
    }
}
