use clap::{Parser, Subcommand};
use dhap_core::CURRENT_SCHEMA_VERSION;
use dhap_export::{HtmlExporter, MarkdownExporter, PreflightValidator};
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
            output_dir,
        } => {
            let (project, frames) = ProjectBundle::load(&project_path)?;
            std::fs::create_dir_all(&output_dir)?;

            let formats: Vec<&str> = format.split(',').map(|s| s.trim()).collect();
            for fmt in formats {
                match fmt {
                    "markdown" | "md" => {
                        let out = MarkdownExporter::export(&project, &frames, &output_dir)?;
                        println!("Exported Markdown: {}", out.display());
                    }
                    "html" => {
                        let out = HtmlExporter::export(&project, &frames, &output_dir)?;
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
    }

    Ok(())
}
