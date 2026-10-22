use clap::{Parser, Subcommand};
use dhap_core::CURRENT_SCHEMA_VERSION;
use dhap_export::{ExportTemplate, HtmlExporter, MarkdownExporter, PreflightValidator};
use dhap_media::{MediaExtractor, MediaProbe, ScreenRecorder};
use dhap_project::ProjectBundle;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dhap")]
#[command(author = "NH Prince Pradhan")]
#[command(version = "0.1.0")]
#[command(about = "Dhap (ধাপ) — Make the steps clear.\nTurn screen recordings and visual workflows into editable documentation.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display system and version diagnostics
    Version,

    /// Inspect a media file container for duration, resolution, and fps
    Probe {
        /// Path to video file
        video_path: PathBuf,
        /// Output formatted JSON
        #[arg(long)]
        json: bool,
    },

    /// Extract candidate keyframes from video at intervals
    ExtractFrames {
        /// Path to video file
        video_path: PathBuf,
        /// Sampling interval in milliseconds
        #[arg(long, default_value_t = 2000)]
        interval_ms: u64,
        /// Destination directory for extracted frames
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Validate a .dhap project bundle for schema compliance and missing assets
    Validate {
        /// Path to .dhap project file
        project_path: PathBuf,
    },

    /// Export documentation from a .dhap project bundle
    Export {
        /// Path to .dhap project file
        project_path: PathBuf,
        /// Export formats comma separated: markdown, html
        #[arg(long, default_value = "markdown,html")]
        format: String,
        /// Export template style: standard, sop, classroom, github
        #[arg(long, default_value = "standard")]
        template: String,
        /// Destination directory for generated guide
        #[arg(long)]
        output_dir: PathBuf,
    },

    /// Record screen or window workflow with timeline step markers
    Record {
        /// Output video file path
        #[arg(short, long, default_value = "dhap_recording.mp4")]
        output: PathBuf,
        /// Recording duration limit in seconds (0 or omitted = wait for Ctrl+C)
        #[arg(short, long, default_value_t = 0)]
        duration: u64,
        /// Screen or window capture device identifier
        #[arg(long, default_value = "desktop")]
        source: String,
        /// Run in headless mock mode for testing and CI validation
        #[arg(long)]
        mock: bool,
    },

    /// Run Model Context Protocol (MCP) JSON-RPC server over stdio for AI agent workflows
    Mcp,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            println!("Dhap Studio CLI: v{}", env!("CARGO_PKG_VERSION"));
            println!("Schema Version: v{}", CURRENT_SCHEMA_VERSION);
            println!("Target Architecture: {}", std::env::consts::ARCH);
            println!("Operating System: {}", std::env::consts::OS);
        }
        Commands::Probe { video_path, json } => {
            let media = MediaProbe::probe(&video_path)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&media)?);
            } else {
                println!("Media File: {}", media.filename);
                println!(
                    "Duration: {} ms ({:.1}s)",
                    media.duration_ms,
                    media.duration_ms as f64 / 1000.0
                );
                println!("Resolution: {}x{}", media.width, media.height);
                println!("Framerate: {:.1} fps", media.fps);
            }
        }
        Commands::ExtractFrames {
            video_path,
            interval_ms,
            output_dir,
        } => {
            std::fs::create_dir_all(&output_dir)?;
            let media = MediaProbe::probe(&video_path)?;
            let timestamps =
                MediaExtractor::suggest_candidate_timestamps(media.duration_ms, interval_ms);

            println!(
                "Extracting {} candidate frames into {}...",
                timestamps.len(),
                output_dir.display()
            );
            for (idx, &ts) in timestamps.iter().enumerate() {
                let frame_bytes = MediaExtractor::extract_frame_bytes(&video_path, ts)?;
                let frame_file = output_dir.join(format!("frame_{:04}_{}ms.png", idx + 1, ts));
                std::fs::write(&frame_file, frame_bytes)?;
            }
            println!("Successfully extracted {} frames.", timestamps.len());
        }
        Commands::Validate { project_path } => {
            let (project, frames) = ProjectBundle::load(&project_path)?;
            let warnings = PreflightValidator::validate(&project, &frames)?;

            println!("Project: {}", project.title);
            println!("Schema Version: {}", project.schema_version);
            println!("Total Steps: {}", project.steps.len());
            println!("Packaged Assets: {}", frames.len());

            if warnings.is_empty() {
                println!("Validation PASSED: 0 errors, 0 warnings.");
            } else {
                println!("Validation PASSED with {} warnings:", warnings.len());
                for w in warnings {
                    println!("  - [WARNING] {}", w);
                }
            }
        }
        Commands::Export {
            project_path,
            format,
            template,
            output_dir,
        } => {
            let (project, frames) = ProjectBundle::load(&project_path)?;
            std::fs::create_dir_all(&output_dir)?;

            let export_template = template
                .parse::<ExportTemplate>()
                .unwrap_or(ExportTemplate::StandardGuide);

            let formats: Vec<&str> = format.split(',').map(|s| s.trim()).collect();
            for fmt in formats {
                match fmt {
                    "markdown" | "md" => {
                        let out = MarkdownExporter::export_with_template(
                            &project,
                            &frames,
                            &output_dir,
                            export_template,
                        )?;
                        println!("Exported Markdown: {}", out.display());
                    }
                    "html" => {
                        let out = HtmlExporter::export_with_template(
                            &project,
                            &frames,
                            &output_dir,
                            export_template,
                        )?;
                        println!("Exported HTML: {}", out.display());
                    }
                    other => {
                        eprintln!("Unknown export format: {}", other);
                    }
                }
            }
            println!("Export completed successfully.");
        }
        Commands::Record {
            output,
            duration,
            source,
            mock,
        } => {
            println!("--- Dhap Screen Recorder ---");
            println!("Output path: {}", output.display());
            println!("Source capture: {}", source);

            if mock {
                println!("Running in simulated/mock capture mode...");
                let mut recording = ScreenRecorder::start_mock_recording(&output, &source);
                println!("Recording started (Session ID: {}).", recording.session.id);

                let wait_duration = if duration > 0 { duration } else { 2 };
                for sec in 1..=wait_duration {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let ts = recording.mark_step(format!("Marker at {}s", sec));
                    println!("  [Step Marked] {}ms - Marker at {}s", ts, sec);
                }

                let session = recording.stop()?;
                println!("Recording finished successfully.");
                println!("Final Duration: {}ms", session.duration_ms);
                println!("Step Markers: {}", session.markers.len());
            } else {
                let recording = ScreenRecorder::start_recording(&output, &source)?;
                println!("Recording live (Session ID: {})...", recording.session.id);
                println!("Press Ctrl+C to stop recording and finalize media.");

                if duration > 0 {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(duration)) => {
                            println!("Duration reached ({}s). Stopping...", duration);
                        }
                        _ = tokio::signal::ctrl_c() => {
                            println!("\nCtrl+C detected. Finalizing recording...");
                        }
                    }
                } else {
                    let _ = tokio::signal::ctrl_c().await;
                    println!("\nCtrl+C detected. Finalizing recording...");
                }

                let session = recording.stop()?;
                println!("Recording finalized: {}", output.display());
                println!("Duration: {:.1}s", session.duration_ms as f64 / 1000.0);
            }
        }
        Commands::Mcp => {
            run_mcp_server().await?;
        }
    }

    Ok(())
}

async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parsed: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = parsed.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = parsed.get("method").and_then(|m| m.as_str()).unwrap_or("");

        let response = match method {
            "initialize" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "dhap",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    }
                })
            }
            "notifications/initialized" => continue,
            "ping" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {}
                })
            }
            "tools/list" => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "dhap_validate_project",
                                "description": "Validates a .dhap project bundle for schema compliance and missing assets",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "project_path": { "type": "string", "description": "Absolute path to .dhap project" }
                                    },
                                    "required": ["project_path"]
                                }
                            },
                            {
                                "name": "dhap_inspect_project",
                                "description": "Inspects and returns metadata, steps, and frames of a .dhap project",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "project_path": { "type": "string", "description": "Absolute path to .dhap project" }
                                    },
                                    "required": ["project_path"]
                                }
                            },
                            {
                                "name": "dhap_scan_secrets",
                                "description": "Heuristically scans text or project steps for unredacted credentials/tokens",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "text": { "type": "string", "description": "Text content to scan" }
                                    },
                                    "required": ["text"]
                                }
                            },
                            {
                                "name": "dhap_export_guide",
                                "description": "Exports a .dhap project into HTML or Markdown using a specified template",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "project_path": { "type": "string", "description": "Absolute path to .dhap file" },
                                        "format": { "type": "string", "description": "markdown, html, or both" },
                                        "template": { "type": "string", "description": "standard, sop, classroom, github" },
                                        "output_dir": { "type": "string", "description": "Destination output directory" }
                                    },
                                    "required": ["project_path", "output_dir"]
                                }
                            }
                        ]
                    }
                })
            }
            "tools/call" => {
                let params = parsed
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                match tool_name {
                    "dhap_scan_secrets" => {
                        let text = args.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        let detected = PreflightValidator::scan_secrets(text);
                        let text_res = if detected.is_empty() {
                            "No unredacted secrets or credentials detected.".to_string()
                        } else {
                            format!(
                                "Found {} potential secret(s):\n{}",
                                detected.len(),
                                detected
                                    .into_iter()
                                    .map(|(t, s)| format!("- {}: '{}'", t, s))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )
                        };
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [{ "type": "text", "text": text_res }]
                            }
                        })
                    }
                    "dhap_validate_project" => {
                        let path_str = args
                            .get("project_path")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        match ProjectBundle::load(std::path::Path::new(path_str)) {
                            Ok((proj, frames)) => {
                                match PreflightValidator::validate(&proj, &frames) {
                                    Ok(warnings) => {
                                        let text_res = if warnings.is_empty() {
                                            format!("Validation passed for '{}'. 0 warnings, {} steps, {} assets.", proj.title, proj.steps.len(), frames.len())
                                        } else {
                                            format!(
                                                "Validation passed with {} warnings:\n{}",
                                                warnings.len(),
                                                warnings
                                                    .iter()
                                                    .map(|w| format!("- {}", w))
                                                    .collect::<Vec<_>>()
                                                    .join("\n")
                                            )
                                        };
                                        serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "result": { "content": [{ "type": "text", "text": text_res }] }
                                        })
                                    }
                                    Err(e) => {
                                        serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "result": { "content": [{ "type": "text", "text": format!("Validation failed: {}", e) }], "isError": true }
                                        })
                                    }
                                }
                            }
                            Err(e) => {
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": format!("Failed to load project: {}", e) }], "isError": true }
                                })
                            }
                        }
                    }
                    "dhap_inspect_project" => {
                        let path_str = args
                            .get("project_path")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        match ProjectBundle::load(std::path::Path::new(path_str)) {
                            Ok((proj, frames)) => {
                                let summary = serde_json::json!({
                                    "id": proj.id,
                                    "title": proj.title,
                                    "description": proj.description,
                                    "step_count": proj.steps.len(),
                                    "frame_count": frames.len(),
                                    "steps": proj.steps.iter().map(|s| serde_json::json!({
                                        "order": s.order,
                                        "title": s.title,
                                        "instruction": s.instruction,
                                        "expected_result": s.expected_result,
                                        "frame_count": s.frame_ids.len()
                                    })).collect::<Vec<_>>()
                                });
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": serde_json::to_string_pretty(&summary).unwrap() }] }
                                })
                            }
                            Err(e) => {
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": format!("Failed to inspect project: {}", e) }], "isError": true }
                                })
                            }
                        }
                    }
                    "dhap_export_guide" => {
                        let path_str = args
                            .get("project_path")
                            .and_then(|p| p.as_str())
                            .unwrap_or("");
                        let out_dir_str = args
                            .get("output_dir")
                            .and_then(|p| p.as_str())
                            .unwrap_or("exports");
                        let tpl_str = args
                            .get("template")
                            .and_then(|t| t.as_str())
                            .unwrap_or("standard");
                        let fmt_str = args
                            .get("format")
                            .and_then(|f| f.as_str())
                            .unwrap_or("markdown,html");

                        let template = tpl_str
                            .parse::<ExportTemplate>()
                            .unwrap_or(ExportTemplate::StandardGuide);
                        let out_dir = std::path::Path::new(out_dir_str);
                        let _ = std::fs::create_dir_all(out_dir);

                        match ProjectBundle::load(std::path::Path::new(path_str)) {
                            Ok((proj, frames)) => {
                                let mut exported = Vec::new();
                                if fmt_str.contains("markdown") || fmt_str.contains("md") {
                                    if let Ok(p) = MarkdownExporter::export_with_template(
                                        &proj, &frames, out_dir, template,
                                    ) {
                                        exported.push(p.display().to_string());
                                    }
                                }
                                if fmt_str.contains("html") {
                                    if let Ok(p) = HtmlExporter::export_with_template(
                                        &proj, &frames, out_dir, template,
                                    ) {
                                        exported.push(p.display().to_string());
                                    }
                                }
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": format!("Exported {} file(s):\n{}", exported.len(), exported.join("\n")) }] }
                                })
                            }
                            Err(e) => {
                                serde_json::json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": format!("Failed to export project: {}", e) }], "isError": true }
                                })
                            }
                        }
                    }
                    other => {
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32601, "message": format!("Tool not found: {}", other) }
                        })
                    }
                }
            }
            _ => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                })
            }
        };

        let resp_str = serde_json::to_string(&response).unwrap();
        writeln!(stdout, "{}", resp_str)?;
        stdout.flush()?;
    }

    Ok(())
}
