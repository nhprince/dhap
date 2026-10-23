export type ProjectId = string;
export type StepId = string;
export type FrameId = string;
export type AnnotationId = string;
export type RedactionId = string;
export type MediaId = string;

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type AnnotationKind =
  | { type: 'CalloutBadge'; data: { number: number } }
  | { type: 'Arrow'; data: { to_x: number; to_y: number } }
  | { type: 'Rectangle' }
  | { type: 'Highlight' }
  | { type: 'Text'; data: { label: string } };

export interface Annotation {
  id: AnnotationId;
  kind: AnnotationKind;
  rect: Rect;
  color_hex: string;
}

export type RedactionStyle =
  | { style: 'Solid'; params: { color_hex: string } }
  | { style: 'Pixelate'; params: { block_size: number } }
  | { style: 'Blur'; params: { radius: number } };

export interface Redaction {
  id: RedactionId;
  rect: Rect;
  style: RedactionStyle;
}

export interface Frame {
  id: FrameId;
  media_id?: MediaId;
  timestamp_ms: number;
  relative_path: string;
  data_url?: string;
  width: number;
  height: number;
  annotations: Annotation[];
  redactions: Redaction[];
}

export type StepStatus = 'draft' | 'reviewed' | 'approved';

export type ActionVerb =
  | 'click'
  | 'type'
  | 'select'
  | 'verify'
  | 'navigate'
  | 'shortcut'
  | 'command'
  | 'wait';

export interface CodeSnippet {
  language: string;
  code: string;
}

export interface ChecklistItem {
  id: string;
  text: string;
  completed?: boolean;
}

export interface GuideStep {
  id: StepId;
  order: number;
  title: string;
  instruction: string;
  expected_result?: string;
  notes?: string;
  frame_ids: FrameId[];
  source_timestamp_ms?: number;
  status: StepStatus;
  action_kind?: ActionVerb;
  code_snippet?: CodeSnippet;
  checklist?: ChecklistItem[];
  tags?: string[];
}

export interface SourceMedia {
  id: MediaId;
  filename: string;
  duration_ms: number;
  width: number;
  height: number;
  fps: number;
  format: string;
}

export interface ExportSettings {
  default_format: string;
  bake_redactions: boolean;
  include_timestamps: boolean;
  output_dir: string;
}

export interface ProjectSnapshot {
  id: string;
  label: string;
  created_at: string;
  step_count: number;
  titles: string[];
}

export interface Project {
  id: ProjectId;
  title: string;
  description?: string;
  schema_version: number;
  created_at: string;
  updated_at: string;
  sources: SourceMedia[];
  frames: Frame[];
  steps: GuideStep[];
  export_settings: ExportSettings;
  snapshots?: ProjectSnapshot[];
}

export type LayoutMode = 'guide' | 'timeline' | 'preview' | 'presentation';
