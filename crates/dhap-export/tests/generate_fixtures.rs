use dhap_core::{Annotation, AnnotationKind, Frame, Project, Rect, Redaction, RedactionStyle};
use dhap_media::SyntheticMedia;
use dhap_project::ProjectBundle;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[test]
fn generate_repo_sample_fixture() {
    let fixtures_dir = Path::new("../../fixtures");
    if !fixtures_dir.exists() {
        let _ = fs::create_dir_all(fixtures_dir);
    }

    let sample_path = fixtures_dir.join("sample_guide.dhap");

    let mut project = Project::new("Deploying with Dhap");
    project.description = Some(
        "A sample guide demonstrating screen recording extraction, step authoring, callouts, and irreversible redaction.".to_string(),
    );

    let frame1_id = "frame_01H001".to_string();
    let frame1_rel = "frames/frame_01.png".to_string();
    let frame2_id = "frame_01H002".to_string();
    let frame2_rel = "frames/frame_02.png".to_string();

    let img1 =
        SyntheticMedia::generate_test_image(800, 500, "1. Terminal Setup", [15, 23, 42, 255]);
    let img2 =
        SyntheticMedia::generate_test_image(800, 500, "2. Cloud Dashboard", [30, 41, 59, 255]);

    let mut raw_frames = HashMap::new();
    raw_frames.insert(frame1_rel.clone(), img1);
    raw_frames.insert(frame2_rel.clone(), img2);

    project.frames.push(Frame {
        id: frame1_id.clone(),
        media_id: None,
        timestamp_ms: 1000,
        relative_path: frame1_rel,
        width: 800,
        height: 500,
        annotations: vec![Annotation {
            id: "anno_1".to_string(),
            kind: AnnotationKind::CalloutBadge { number: 1 },
            rect: Rect {
                x: 100,
                y: 100,
                width: 36,
                height: 36,
            },
            color_hex: "#F97316".to_string(),
        }],
        redactions: vec![Redaction {
            id: "redact_secret".to_string(),
            rect: Rect {
                x: 30,
                y: 30,
                width: 140,
                height: 25,
            },
            style: RedactionStyle::Solid {
                color_hex: "#000000".to_string(),
            },
        }],
    });

    project.frames.push(Frame {
        id: frame2_id.clone(),
        media_id: None,
        timestamp_ms: 4500,
        relative_path: frame2_rel,
        width: 800,
        height: 500,
        annotations: vec![Annotation {
            id: "anno_2".to_string(),
            kind: AnnotationKind::Rectangle,
            rect: Rect {
                x: 200,
                y: 150,
                width: 250,
                height: 120,
            },
            color_hex: "#22C55E".to_string(),
        }],
        redactions: Vec::new(),
    });

    project.add_step(
        "Clone and Build",
        "Clone the repository and run `cargo build --workspace`.",
        Some(frame1_id),
        Some(1000),
    );

    project.add_step(
        "Verify Dashboard",
        "Open the local dashboard and verify all health checks pass.",
        Some(frame2_id),
        Some(4500),
    );

    let _ = ProjectBundle::save(&project, &raw_frames, &sample_path);
}
