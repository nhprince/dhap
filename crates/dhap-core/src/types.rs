use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Unique type-safe identifiers
pub type ProjectId = String;
pub type StepId = String;
pub type FrameId = String;
pub type AnnotationId = String;
pub type RedactionId = String;
pub type MediaId = String;

pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum AnnotationKind {
    CalloutBadge { number: u32 },
    Arrow { to_x: u32, to_y: u32 },
    Rectangle,
    Highlight,
    Text { label: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Annotation {
    pub id: AnnotationId,
    pub kind: AnnotationKind,
    pub rect: Rect,
    pub color_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "style", content = "params")]
pub enum RedactionStyle {
    Solid { color_hex: String },
    Pixelate { block_size: u32 },
    Blur { radius: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Redaction {
    pub id: RedactionId,
    pub rect: Rect,
    pub style: RedactionStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    pub id: FrameId,
    pub media_id: Option<MediaId>,
    pub timestamp_ms: u64,
    pub relative_path: String,
    pub width: u32,
    pub height: u32,
    pub annotations: Vec<Annotation>,
    pub redactions: Vec<Redaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Draft,
    Reviewed,
    Approved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuideStep {
    pub id: StepId,
    pub order: u32,
    pub title: String,
    pub instruction: String,
    pub expected_result: Option<String>,
    pub notes: Option<String>,
    pub frame_ids: Vec<FrameId>,
    pub source_timestamp_ms: Option<u64>,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureMarker {
    pub timestamp_ms: u64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureSession {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: u64,
    pub source_name: String,
    pub markers: Vec<CaptureMarker>,
    pub output_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMedia {
    pub id: MediaId,
    pub filename: String,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportSettings {
    pub default_format: String,
    pub bake_redactions: bool,
    pub include_timestamps: bool,
    pub output_dir: String,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_format: "markdown".to_string(),
            bake_redactions: true,
            include_timestamps: true,
            output_dir: "exports".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub title: String,
    pub description: Option<String>,
    pub schema_version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sources: Vec<SourceMedia>,
    pub frames: Vec<Frame>,
    pub steps: Vec<GuideStep>,
    pub export_settings: ExportSettings,
}

impl Project {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id("proj"),
            title: title.into(),
            description: None,
            schema_version: CURRENT_SCHEMA_VERSION,
            created_at: now,
            updated_at: now,
            sources: Vec::new(),
            frames: Vec::new(),
            steps: Vec::new(),
            export_settings: ExportSettings::default(),
        }
    }

    pub fn add_step(
        &mut self,
        title: impl Into<String>,
        instruction: impl Into<String>,
        frame_id: Option<FrameId>,
        timestamp_ms: Option<u64>,
    ) -> StepId {
        let id = generate_id("step");
        let order = self.steps.len() as u32 + 1;
        let mut frame_ids = Vec::new();
        if let Some(fid) = frame_id {
            frame_ids.push(fid);
        }

        self.steps.push(GuideStep {
            id: id.clone(),
            order,
            title: title.into(),
            instruction: instruction.into(),
            expected_result: None,
            notes: None,
            frame_ids,
            source_timestamp_ms: timestamp_ms,
            status: StepStatus::Draft,
        });

        self.updated_at = Utc::now();
        id
    }

    pub fn get_frame(&self, frame_id: &str) -> Option<&Frame> {
        self.frames.iter().find(|f| f.id == frame_id)
    }

    pub fn get_frame_mut(&mut self, frame_id: &str) -> Option<&mut Frame> {
        self.frames.iter_mut().find(|f| f.id == frame_id)
    }
}
