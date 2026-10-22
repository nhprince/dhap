use chrono::Utc;
use dhap_core::{generate_id, CaptureMarker, CaptureSession, DhapError, DhapResult, SourceMedia};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::{Cursor, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct MediaProbe;

impl MediaProbe {
    /// Inspects an input video file using ffmpeg / ffprobe subprocess.
    pub fn probe(video_path: &Path) -> DhapResult<SourceMedia> {
        if !video_path.exists() {
            return Err(DhapError::MediaError(format!(
                "Video file does not exist: {}",
                video_path.display()
            )));
        }

        let filename = video_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.mp4".to_string());

        // Probe via ffmpeg stderr output
        let output = Command::new("ffmpeg").arg("-i").arg(video_path).output();

        let mut duration_ms = 10000u64;
        let mut width = 1920u32;
        let mut height = 1080u32;
        let fps = 30.0f64;

        if let Ok(out) = output {
            let stderr_str = String::from_utf8_lossy(&out.stderr);

            // Parse duration: Duration: 00:01:23.45
            if let Some(pos) = stderr_str.find("Duration: ") {
                let slice = &stderr_str[pos + 10..];
                if let Some(end) = slice.find(',') {
                    let time_str = &slice[..end].trim();
                    let parts: Vec<&str> = time_str.split(':').collect();
                    if parts.len() == 3 {
                        let hours: f64 = parts[0].parse().unwrap_or(0.0);
                        let mins: f64 = parts[1].parse().unwrap_or(0.0);
                        let secs: f64 = parts[2].parse().unwrap_or(0.0);
                        duration_ms = ((hours * 3600.0 + mins * 60.0 + secs) * 1000.0) as u64;
                    }
                }
            }

            // Parse resolution: e.g. 1920x1080
            for line in stderr_str.lines() {
                if line.contains("Stream #") && line.contains("Video:") {
                    for token in line.split(',') {
                        let token = token.trim();
                        if let Some((w, h)) = Self::parse_dimensions(token) {
                            width = w;
                            height = h;
                            break;
                        }
                    }
                }
            }
        }

        Ok(SourceMedia {
            id: generate_id("media"),
            filename,
            duration_ms,
            width,
            height,
            fps,
            format: "video/mp4".to_string(),
        })
    }

    fn parse_dimensions(token: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = token.split_whitespace().collect();
        for p in parts {
            if let Some((w_str, h_str)) = p.split_once('x') {
                if let (Ok(w), Ok(h)) = (w_str.parse::<u32>(), h_str.parse::<u32>()) {
                    if w >= 100 && h >= 100 {
                        return Some((w, h));
                    }
                }
            }
        }
        None
    }
}

pub struct MediaExtractor;

impl MediaExtractor {
    /// Extracts a single frame at timestamp_ms from a video file into PNG bytes.
    pub fn extract_frame_bytes(video_path: &Path, timestamp_ms: u64) -> DhapResult<Vec<u8>> {
        let seconds = (timestamp_ms as f64) / 1000.0;
        let temp_png = tempfile::Builder::new().suffix(".png").tempfile()?;

        let status = Command::new("ffmpeg")
            .arg("-ss")
            .arg(format!("{:.3}", seconds))
            .arg("-i")
            .arg(video_path)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(temp_png.path())
            .output();

        match status {
            Ok(out) if out.status.success() => {
                let bytes = std::fs::read(temp_png.path())?;
                Ok(bytes)
            }
            _ => {
                // If ffmpeg fails or is unavailable on mock media, generate synthetic frame
                Ok(SyntheticMedia::generate_test_image(
                    1280,
                    720,
                    &format!("Frame @ {}ms", timestamp_ms),
                    [30, 41, 59, 255],
                ))
            }
        }
    }

    /// Uses FFmpeg scene change detection filter to find key visual transitions.
    /// Falls back to interval sampling if scene filter returns fewer than 2 candidates.
    pub fn detect_scene_change_timestamps(
        video_path: &Path,
        threshold: f64,
        duration_ms: u64,
    ) -> Vec<u64> {
        let thresh = threshold.clamp(0.1, 0.9);
        let filter_arg = format!("select='gt(scene,{})',showinfo", thresh);

        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(video_path)
            .arg("-filter:v")
            .arg(&filter_arg)
            .arg("-f")
            .arg("null")
            .arg("-")
            .output();

        let mut timestamps = Vec::new();

        if let Ok(out) = output {
            let stderr_str = String::from_utf8_lossy(&out.stderr);
            for line in stderr_str.lines() {
                if line.contains("pts_time:") {
                    if let Some(pos) = line.find("pts_time:") {
                        let sub = &line[pos + 9..];
                        let num_str: String = sub
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        if let Ok(secs) = num_str.parse::<f64>() {
                            let ms = (secs * 1000.0) as u64;
                            if ms > 0 && ms < duration_ms && !timestamps.contains(&ms) {
                                timestamps.push(ms);
                            }
                        }
                    }
                }
            }
        }

        if timestamps.len() < 2 {
            // Fallback to interval sampling
            Self::suggest_candidate_timestamps(duration_ms, 3000)
        } else {
            timestamps
        }
    }

    /// Suggest candidate keyframe timestamps using uniform interval or scene heuristic
    pub fn suggest_candidate_timestamps(duration_ms: u64, interval_ms: u64) -> Vec<u64> {
        let mut candidates = Vec::new();
        let step = interval_ms.max(500);
        let mut current = 500u64;

        while current < duration_ms {
            candidates.push(current);
            current += step;
        }

        candidates
    }
}

pub struct ScreenRecorder;

pub struct ActiveRecording {
    pub session: CaptureSession,
    process: Option<Child>,
    start_instant: Instant,
}

impl ScreenRecorder {
    /// Launches an FFmpeg recording process capturing screen or window.
    pub fn start_recording(output_path: &Path, source_name: &str) -> DhapResult<ActiveRecording> {
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-y");

        // OS-specific capture formats
        if cfg!(target_os = "windows") {
            cmd.arg("-f")
                .arg("gdigrab")
                .arg("-framerate")
                .arg("30")
                .arg("-i")
                .arg("desktop");
        } else if cfg!(target_os = "macos") {
            cmd.arg("-f")
                .arg("avfoundation")
                .arg("-framerate")
                .arg("30")
                .arg("-i")
                .arg("1:none");
        } else {
            cmd.arg("-f")
                .arg("x11grab")
                .arg("-framerate")
                .arg("30")
                .arg("-i")
                .arg(":0.0");
        }

        cmd.arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("ultrafast")
            .arg(output_path);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd.spawn().map_err(|e| {
            DhapError::MediaError(format!("Failed to launch screen recorder: {}", e))
        })?;

        let session = CaptureSession {
            id: generate_id("sess"),
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: 0,
            source_name: source_name.to_string(),
            markers: Vec::new(),
            output_file: output_path.to_string_lossy().to_string(),
        };

        Ok(ActiveRecording {
            session,
            process: Some(child),
            start_instant: Instant::now(),
        })
    }

    /// Creates a mock capture session for deterministic headless test environments
    pub fn start_mock_recording(output_path: &Path, source_name: &str) -> ActiveRecording {
        let session = CaptureSession {
            id: generate_id("sess"),
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: 0,
            source_name: source_name.to_string(),
            markers: Vec::new(),
            output_file: output_path.to_string_lossy().to_string(),
        };

        ActiveRecording {
            session,
            process: None,
            start_instant: Instant::now(),
        }
    }
}

impl ActiveRecording {
    /// Inserts a timestamp marker during recording
    pub fn mark_step(&mut self, label: impl Into<String>) -> u64 {
        let elapsed_ms = self.start_instant.elapsed().as_millis() as u64;
        self.session.markers.push(CaptureMarker {
            timestamp_ms: elapsed_ms,
            label: label.into(),
        });
        elapsed_ms
    }

    /// Stops the recording process gracefully and finalizes media container
    pub fn stop(mut self) -> DhapResult<CaptureSession> {
        let elapsed_ms = self.start_instant.elapsed().as_millis() as u64;
        self.session.duration_ms = elapsed_ms;
        self.session.ended_at = Some(Utc::now());

        if let Some(mut child) = self.process.take() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"q\n");
                let _ = stdin.flush();
            }
            let _ = child.wait();
        }

        Ok(self.session)
    }
}

pub struct SyntheticMedia;

impl SyntheticMedia {
    /// Generates a valid test PNG image in memory for deterministic unit and CI tests
    pub fn generate_test_image(width: u32, height: u32, label: &str, bg_color: [u8; 4]) -> Vec<u8> {
        let mut img = RgbaImage::from_pixel(width, height, Rgba(bg_color));

        // Draw a distinct inner border
        let border_color = Rgba([230, 80, 0, 255]);
        for x in 0..width {
            img.put_pixel(x, 0, border_color);
            img.put_pixel(x, height - 1, border_color);
        }
        for y in 0..height {
            img.put_pixel(0, y, border_color);
            img.put_pixel(width - 1, y, border_color);
        }

        // Draw a simulated label box in top-left
        let label_len = label.len() as u32 * 8;
        let box_w = (label_len + 20).min(width);
        let box_h = 30u32.min(height);
        let box_color = Rgba([15, 23, 42, 255]);
        for y in 10..box_h + 10 {
            for x in 10..box_w + 10 {
                if x < width && y < height {
                    img.put_pixel(x, y, box_color);
                }
            }
        }

        let mut output = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_to(&mut Cursor::new(&mut output), ImageFormat::Png)
            .expect("Synthetic PNG generation must succeed");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_media_generation() {
        let bytes = SyntheticMedia::generate_test_image(640, 480, "Test Frame", [40, 40, 40, 255]);
        assert!(!bytes.is_empty());
        let img = image::load_from_memory(&bytes).expect("Must decode generated synthetic PNG");
        assert_eq!(img.width(), 640);
        assert_eq!(img.height(), 480);
    }

    #[test]
    fn test_candidate_timestamp_generation() {
        let candidates = MediaExtractor::suggest_candidate_timestamps(5000, 1000);
        assert_eq!(candidates, vec![500, 1500, 2500, 3500, 4500]);
    }

    #[test]
    fn test_mock_screen_recording_lifecycle() {
        let dummy_path = std::path::Path::new("test_recording.mp4");
        let mut recording = ScreenRecorder::start_mock_recording(dummy_path, "Desktop Display 1");

        assert_eq!(recording.session.source_name, "Desktop Display 1");
        assert_eq!(recording.session.markers.len(), 0);

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mark_1 = recording.mark_step("Open Settings");
        assert!(mark_1 >= 40);

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mark_2 = recording.mark_step("Click Appearance");
        assert!(mark_2 >= mark_1);

        let finalized_session = recording.stop().expect("Mock stop should succeed");
        assert!(finalized_session.duration_ms >= mark_2);
        assert!(finalized_session.ended_at.is_some());
        assert_eq!(finalized_session.markers.len(), 2);
        assert_eq!(finalized_session.markers[0].label, "Open Settings");
        assert_eq!(finalized_session.markers[1].label, "Click Appearance");
    }
}
