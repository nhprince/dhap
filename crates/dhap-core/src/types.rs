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
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Click,
    Type,
    Select,
    Verify,
    Navigate,
    KeyShortcut,
    Command,
    Wait,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeSnippet {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub completed: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_kind: Option<ActionKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_snippet: Option<CodeSnippet>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checklist: Vec<ChecklistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
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
pub struct ProjectSnapshot {
    pub id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub step_count: usize,
    pub titles: Vec<String>,
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
    #[serde(default)]
    pub snapshots: Vec<ProjectSnapshot>,
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
            snapshots: Vec::new(),
        }
    }

    /// Creates a named snapshot of current project step state
    pub fn create_snapshot(&mut self, label: impl Into<String>) -> String {
        let snap_id = generate_id("snap");
        let titles = self.steps.iter().map(|s| s.title.clone()).collect();
        self.snapshots.push(ProjectSnapshot {
            id: snap_id.clone(),
            label: label.into(),
            created_at: Utc::now(),
            step_count: self.steps.len(),
            titles,
        });
        self.updated_at = Utc::now();
        snap_id
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
            action_kind: None,
            code_snippet: None,
            checklist: Vec::new(),
            tags: Vec::new(),
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

    /// Reorders a step to a new 0-based index and recalibrates all step.order values
    pub fn reorder_step(&mut self, step_id: &str, new_index: usize) -> bool {
        let current_pos = match self.steps.iter().position(|s| s.id == step_id) {
            Some(p) => p,
            None => return false,
        };

        if current_pos == new_index || new_index >= self.steps.len() {
            return false;
        }

        let step = self.steps.remove(current_pos);
        self.steps.insert(new_index, step);

        for (idx, s) in self.steps.iter_mut().enumerate() {
            s.order = idx as u32 + 1;
        }

        self.updated_at = Utc::now();
        true
    }

    /// Moves a step up by one position
    pub fn move_step_up(&mut self, step_id: &str) -> bool {
        let pos = match self.steps.iter().position(|s| s.id == step_id) {
            Some(p) => p,
            None => return false,
        };

        if pos == 0 {
            return false;
        }

        self.reorder_step(step_id, pos - 1)
    }

    /// Moves a step down by one position
    pub fn move_step_down(&mut self, step_id: &str) -> bool {
        let pos = match self.steps.iter().position(|s| s.id == step_id) {
            Some(p) => p,
            None => return false,
        };

        if pos + 1 >= self.steps.len() {
            return false;
        }

        self.reorder_step(step_id, pos + 1)
    }

    /// Search steps matching title, instruction, notes or tags
    pub fn search_steps(&self, query: &str) -> Vec<&GuideStep> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.steps.iter().collect();
        }

        self.steps
            .iter()
            .filter(|s| {
                s.title.to_lowercase().contains(&q)
                    || s.instruction.to_lowercase().contains(&q)
                    || s.expected_result
                        .as_deref()
                        .map(|er| er.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || s.notes
                        .as_deref()
                        .map(|n| n.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Calculates total duration spanned across source timestamps
    pub fn total_duration_ms(&self) -> u64 {
        let mut max_ts = 0u64;
        for s in &self.steps {
            if let Some(ts) = s.source_timestamp_ms {
                if ts > max_ts {
                    max_ts = ts;
                }
            }
        }
        for f in &self.frames {
            if f.timestamp_ms > max_ts {
                max_ts = f.timestamp_ms;
            }
        }
        for src in &self.sources {
            if src.duration_ms > max_ts {
                max_ts = src.duration_ms;
            }
        }
        max_ts
    }

    /// Computes rich analytics and project statistics
    pub fn statistics(&self) -> ProjectStatistics {
        let total_words: usize = self
            .steps
            .iter()
            .map(|s| {
                s.title.split_whitespace().count()
                    + s.instruction.split_whitespace().count()
                    + s.expected_result
                        .as_deref()
                        .map(|er| er.split_whitespace().count())
                        .unwrap_or(0)
                    + s.notes
                        .as_deref()
                        .map(|n| n.split_whitespace().count())
                        .unwrap_or(0)
            })
            .sum();

        let annotation_count: usize = self.frames.iter().map(|f| f.annotations.len()).sum();
        let redaction_count: usize = self.frames.iter().map(|f| f.redactions.len()).sum();

        // Average reading speed: 200 words per minute + 30 seconds per screenshot/step
        let reading_time_from_words = (total_words as f64) / 200.0;
        let reading_time_from_steps = (self.steps.len() as f64) * 0.5;
        let estimated_reading_time_mins =
            (reading_time_from_words + reading_time_from_steps).ceil() as usize;

        ProjectStatistics {
            step_count: self.steps.len(),
            frame_count: self.frames.len(),
            annotation_count,
            redaction_count,
            total_words,
            estimated_reading_time_mins: estimated_reading_time_mins.max(1),
            total_duration_ms: self.total_duration_ms(),
        }
    }

    /// Compares this project against another project, generating a structured diff
    pub fn compute_diff(&self, other: &Project) -> ProjectDiff {
        let mut added_steps = Vec::new();
        let mut removed_steps = Vec::new();
        let mut modified_steps = Vec::new();

        for other_step in &other.steps {
            if !self.steps.iter().any(|s| s.id == other_step.id) {
                added_steps.push(other_step.clone());
            }
        }

        for this_step in &self.steps {
            match other.steps.iter().find(|s| s.id == this_step.id) {
                None => {
                    removed_steps.push(this_step.clone());
                }
                Some(matching) => {
                    let title_changed = this_step.title != matching.title;
                    let instruction_changed = this_step.instruction != matching.instruction;
                    let frame_changed = this_step.frame_ids != matching.frame_ids;

                    if title_changed || instruction_changed || frame_changed {
                        modified_steps.push(StepDiff {
                            step_id: this_step.id.clone(),
                            title_before: Some(this_step.title.clone()),
                            title_after: Some(matching.title.clone()),
                            instruction_changed,
                            frame_changed,
                            change_type: StepChangeType::Modified,
                        });
                    }
                }
            }
        }

        let summary = format!(
            "Diff summary: +{} added, -{} removed, ~{} modified steps",
            added_steps.len(),
            removed_steps.len(),
            modified_steps.len()
        );

        ProjectDiff {
            base_project_id: self.id.clone(),
            target_project_id: other.id.clone(),
            added_steps,
            removed_steps,
            modified_steps,
            total_steps_before: self.steps.len(),
            total_steps_after: other.steps.len(),
            summary,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectStatistics {
    pub step_count: usize,
    pub frame_count: usize,
    pub annotation_count: usize,
    pub redaction_count: usize,
    pub total_words: usize,
    pub estimated_reading_time_mins: usize,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepChangeType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepDiff {
    pub step_id: StepId,
    pub title_before: Option<String>,
    pub title_after: Option<String>,
    pub instruction_changed: bool,
    pub frame_changed: bool,
    pub change_type: StepChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectDiff {
    pub base_project_id: ProjectId,
    pub target_project_id: ProjectId,
    pub added_steps: Vec<GuideStep>,
    pub removed_steps: Vec<GuideStep>,
    pub modified_steps: Vec<StepDiff>,
    pub total_steps_before: usize,
    pub total_steps_after: usize,
    pub summary: String,
}
