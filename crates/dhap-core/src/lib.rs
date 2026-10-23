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

    #[test]
    fn test_step_reordering() {
        let mut proj = Project::new("Reorder Test");
        let s1 = proj.add_step("Step 1", "First", None, None);
        let s2 = proj.add_step("Step 2", "Second", None, None);
        let s3 = proj.add_step("Step 3", "Third", None, None);

        assert_eq!(proj.steps[0].id, s1);
        assert_eq!(proj.steps[1].id, s2);
        assert_eq!(proj.steps[2].id, s3);

        // Move s3 up
        assert!(proj.move_step_up(&s3));
        assert_eq!(proj.steps[1].id, s3);
        assert_eq!(proj.steps[1].order, 2);
        assert_eq!(proj.steps[2].id, s2);
        assert_eq!(proj.steps[2].order, 3);

        // Move s1 down
        assert!(proj.move_step_down(&s1));
        assert_eq!(proj.steps[0].id, s3);
        assert_eq!(proj.steps[0].order, 1);
        assert_eq!(proj.steps[1].id, s1);
        assert_eq!(proj.steps[1].order, 2);

        // Cannot move first item up
        assert!(!proj.move_step_up(&s3));
        // Cannot move last item down
        assert!(!proj.move_step_down(&s2));
    }

    #[test]
    fn test_project_snapshot() {
        let mut proj = Project::new("Snapshot Test");
        proj.add_step("Step A", "Do A", None, None);
        proj.add_step("Step B", "Do B", None, None);

        let snap_id = proj.create_snapshot("Initial Revision");
        assert_eq!(proj.snapshots.len(), 1);
        assert_eq!(proj.snapshots[0].id, snap_id);
        assert_eq!(proj.snapshots[0].label, "Initial Revision");
        assert_eq!(proj.snapshots[0].step_count, 2);
        assert_eq!(proj.snapshots[0].titles, vec!["Step A", "Step B"]);
    }

    #[test]
    fn test_project_diff_and_statistics() {
        let mut proj1 = Project::new("Version 1");
        let s1 = proj1.add_step(
            "Open Terminal",
            "Launch your favorite shell",
            None,
            Some(1000),
        );
        let _s2 = proj1.add_step("Run Build", "Execute cargo build", None, Some(2000));

        let stats = proj1.statistics();
        assert_eq!(stats.step_count, 2);
        assert!(stats.total_words > 0);
        assert_eq!(stats.total_duration_ms, 2000);

        let mut proj2 = proj1.clone();
        proj2.id = generate_id("proj");
        // Modify step 1
        proj2.steps[0].instruction = "Launch PowerShell as administrator".to_string();
        // Remove step 2
        proj2.steps.retain(|s| s.id == s1);
        // Add step 3
        proj2.add_step(
            "Verify Output",
            "Check target/release folder",
            None,
            Some(3500),
        );

        let diff = proj1.compute_diff(&proj2);
        assert_eq!(diff.added_steps.len(), 1);
        assert_eq!(diff.removed_steps.len(), 1);
        assert_eq!(diff.modified_steps.len(), 1);
        assert_eq!(diff.total_steps_before, 2);
        assert_eq!(diff.total_steps_after, 2);
        assert!(diff.summary.contains("+1 added, -1 removed, ~1 modified"));
    }

    #[test]
    fn test_search_steps() {
        let mut proj = Project::new("Search Test");
        let s1_id = proj.add_step("Configure AWS", "Set your access key", None, None);
        let _s2_id = proj.add_step("Deploy Cluster", "Run terraform apply", None, None);

        // Add a tag to s1
        proj.steps[0].tags.push("cloud".to_string());
        proj.steps[0].tags.push("security".to_string());

        let results = proj.search_steps("aws");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, s1_id);

        let tag_results = proj.search_steps("security");
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].id, s1_id);

        let empty_query = proj.search_steps("");
        assert_eq!(empty_query.len(), 2);
    }
}
