pub mod error;
pub mod types;

pub use error::{DhapError, DhapResult};
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation_and_step_addition() {
        let mut proj = Project::new("Deploying with Dhap");
        assert_eq!(proj.schema_version, 1);
        assert_eq!(proj.steps.len(), 0);

        let step_id = proj.add_step(
            "Install Dependencies",
            "Run cargo build --release to compile artifacts.",
            None,
            Some(1500),
        );

        assert_eq!(proj.steps.len(), 1);
        let step = &proj.steps[0];
        assert_eq!(step.id, step_id);
        assert_eq!(step.title, "Install Dependencies");
        assert_eq!(step.source_timestamp_ms, Some(1500));
        assert_eq!(step.status, StepStatus::Draft);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut proj = Project::new("Serialization Test");
        let fid = generate_id("frame");
        proj.frames.push(Frame {
            id: fid.clone(),
            media_id: None,
            timestamp_ms: 2500,
            relative_path: "frames/sample.png".to_string(),
            width: 1920,
            height: 1080,
            annotations: vec![Annotation {
                id: generate_id("anno"),
                kind: AnnotationKind::CalloutBadge { number: 1 },
                rect: Rect {
                    x: 100,
                    y: 100,
                    width: 50,
                    height: 50,
                },
                color_hex: "#FF5500".to_string(),
            }],
            redactions: vec![Redaction {
                id: generate_id("redact"),
                rect: Rect {
                    x: 200,
                    y: 200,
                    width: 150,
                    height: 40,
                },
                style: RedactionStyle::Solid {
                    color_hex: "#000000".to_string(),
                },
            }],
        });
        proj.add_step(
            "Step with Frame",
            "Check the callout badge",
            Some(fid),
            Some(2500),
        );

        let json = serde_json::to_string_pretty(&proj).expect("Must serialize project");
        let deserialized: Project = serde_json::from_str(&json).expect("Must deserialize project");

        assert_eq!(proj.id, deserialized.id);
        assert_eq!(deserialized.frames.len(), 1);
        assert_eq!(deserialized.frames[0].annotations.len(), 1);
        assert_eq!(deserialized.frames[0].redactions.len(), 1);
    }
}
