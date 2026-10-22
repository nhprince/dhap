use dhap_core::{Annotation, AnnotationKind, Frame, Project, Rect, Redaction, RedactionStyle};
use dhap_export::{HtmlExporter, MarkdownExporter, PreflightValidator};
use dhap_media::SyntheticMedia;
use dhap_project::ProjectBundle;
use image::GenericImageView;
use std::collections::HashMap;
use std::fs;

#[test]
fn test_complete_e2e_workflow_vertical_slice() {
    let temp_dir = tempfile::tempdir().expect("Create temp dir");
    let bundle_path = temp_dir.path().join("deployment_guide.dhap");
    let export_dir = temp_dir.path().join("exported_guide");

    // =========================================================================
    // Stage 1: Create Project and Ingest Frames
    // =========================================================================
    let mut project = Project::new("Production Deployment Protocol");
    project.description =
        Some("Step-by-step verified workflow for deploying services.".to_string());

    let frame1_id = "frame_001".to_string();
    let frame1_rel_path = "frames/frame_001.png".to_string();

    // Generate synthetic test frame with a white background and red marker
    let synthetic_png_bytes =
        SyntheticMedia::generate_test_image(800, 600, "Server Terminal", [255, 255, 255, 255]);

    let mut raw_frames = HashMap::new();
    raw_frames.insert(frame1_rel_path.clone(), synthetic_png_bytes);

    // =========================================================================
    // Stage 2: Add Annotations and Irreversible Redaction
    // =========================================================================
    let frame1 = Frame {
        id: frame1_id.clone(),
        media_id: None,
        timestamp_ms: 2500,
        relative_path: frame1_rel_path,
        width: 800,
        height: 600,
        annotations: vec![Annotation {
            id: "anno_callout_1".to_string(),
            kind: AnnotationKind::CalloutBadge { number: 1 },
            rect: Rect {
                x: 150,
                y: 150,
                width: 40,
                height: 40,
            },
            color_hex: "#F97316".to_string(),
        }],
        redactions: vec![Redaction {
            id: "redact_auth_token".to_string(),
            rect: Rect {
                x: 20,
                y: 20,
                width: 100,
                height: 30,
            },
            style: RedactionStyle::Solid {
                color_hex: "#000000".to_string(),
            },
        }],
    };

    project.frames.push(frame1.clone());

    // Step 1: Login & Config
    let step1_id = project.add_step(
        "Authenticate to Gateway",
        "Open terminal and execute deployment login with your credentials.",
        Some(frame1_id.clone()),
        Some(2500),
    );
    assert_eq!(project.steps.len(), 1);

    // Step 2: Verification
    project.add_step(
        "Verify Healthcheck",
        "Check /healthz endpoint for 200 OK status.",
        None,
        Some(5000),
    );
    assert_eq!(project.steps.len(), 2);

    // =========================================================================
    // Stage 3: Atomic Save to .dhap Archive
    // =========================================================================
    ProjectBundle::save(&project, &raw_frames, &bundle_path).expect("Saving .dhap must succeed");
    assert!(bundle_path.exists());

    // =========================================================================
    // Stage 4: Reopen from Disk and Verify State Integrity
    // =========================================================================
    let (loaded_project, loaded_frames) =
        ProjectBundle::load(&bundle_path).expect("Loading .dhap must succeed");

    assert_eq!(loaded_project.title, "Production Deployment Protocol");
    assert_eq!(loaded_project.steps.len(), 2);
    assert_eq!(loaded_project.steps[0].id, step1_id);
    assert_eq!(loaded_frames.len(), 1);

    // =========================================================================
    // Stage 5: Preflight Validation and Multi-Format Exports
    // =========================================================================
    let warnings = PreflightValidator::validate(&loaded_project, &loaded_frames)
        .expect("Preflight validation must pass");
    assert!(warnings.is_empty(), "Expected 0 preflight warnings");

    // Export Markdown
    let md_file = MarkdownExporter::export(&loaded_project, &loaded_frames, &export_dir)
        .expect("Markdown export must succeed");
    assert!(md_file.exists());
    let md_content = fs::read_to_string(&md_file).unwrap();
    assert!(md_content.contains("# Production Deployment Protocol"));
    assert!(md_content.contains("### Step 1: Authenticate to Gateway"));
    assert!(md_content.contains("assets/step_1_"));

    // Export Standalone HTML
    let html_file = HtmlExporter::export(&loaded_project, &loaded_frames, &export_dir)
        .expect("HTML export must succeed");
    assert!(html_file.exists());
    let html_content = fs::read_to_string(&html_file).unwrap();
    assert!(html_content.contains("<!DOCTYPE html>"));
    assert!(html_content.contains("Production Deployment Protocol - Dhap Guide"));

    // =========================================================================
    // Stage 6: Mathematical Proof of Irreversible Pixel Redaction
    // =========================================================================
    let exported_asset_path = export_dir.join(format!("assets/step_1_{}.png", frame1_id));
    assert!(
        exported_asset_path.exists(),
        "Exported baked asset must exist"
    );

    let baked_img_bytes = fs::read(&exported_asset_path).unwrap();
    let baked_img = image::load_from_memory(&baked_img_bytes).unwrap();

    // Verify that every pixel in the redacted box (x: 20..120, y: 20..50) is solid black (0, 0, 0, 255)
    for y in 20..50 {
        for x in 20..120 {
            let pixel = baked_img.get_pixel(x, y);
            assert_eq!(pixel[0], 0, "Redacted pixel at ({},{}) must have R=0", x, y);
            assert_eq!(pixel[1], 0, "Redacted pixel at ({},{}) must have G=0", x, y);
            assert_eq!(pixel[2], 0, "Redacted pixel at ({},{}) must have B=0", x, y);
            assert_eq!(
                pixel[3], 255,
                "Redacted pixel at ({},{}) must have Alpha=255",
                x, y
            );
        }
    }

    // Verify that unredacted areas remained intact
    let unredacted_pixel = baked_img.get_pixel(300, 300);
    assert_eq!(
        unredacted_pixel[0], 255,
        "Unredacted area must preserve original white background"
    );
}
