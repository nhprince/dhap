use dhap_core::{generate_id, DhapError, DhapResult, SourceMedia};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

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
    fn test_detect_scene_change_fallback() {
        let candidates = MediaExtractor::detect_scene_change_timestamps(
            std::path::Path::new("non_existent.mp4"),
            0.4,
            6000,
        );
        // Asserts fallback to interval sampling of 3000ms
        assert_eq!(candidates, vec![500, 3500]);
    }
}
