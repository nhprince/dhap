use chrono::{DateTime, Utc};
use dhap_core::{DhapError, DhapResult, Project, CURRENT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub app_version: String,
    pub generator: String,
    pub created_at: DateTime<Utc>,
    pub file_count: usize,
}

pub struct ProjectBundle;

impl ProjectBundle {
    /// Save a project and its associated image buffers into a .dhap ZIP archive.
    /// Utilizes an atomic write pattern (temp file -> atomic rename) to avoid corruption.
    pub fn save(
        project: &Project,
        frames: &HashMap<String, Vec<u8>>,
        destination: &Path,
    ) -> DhapResult<()> {
        let parent_dir = destination.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent_dir)?;

        let temp_file = tempfile::Builder::new()
            .prefix("dhap_save_")
            .suffix(".tmp")
            .tempfile_in(parent_dir)?;

        {
            let file = temp_file.as_file();
            let mut zip = ZipWriter::new(BufWriter::new(file));
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            // 1. Write manifest.json
            let manifest = Manifest {
                schema_version: project.schema_version,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                generator: "Dhap Studio 0.1.0".to_string(),
                created_at: Utc::now(),
                file_count: frames.len() + 2,
            };
            zip.start_file("manifest.json", options)
                .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
            let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
            zip.write_all(&manifest_bytes)?;

            // 2. Write project.json
            zip.start_file("project.json", options)
                .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
            let project_bytes = serde_json::to_vec_pretty(project)?;
            zip.write_all(&project_bytes)?;

            // 3. Write frames
            for (rel_path, data) in frames {
                Self::validate_relative_path(rel_path)?;
                zip.start_file(rel_path, options)
                    .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
                zip.write_all(data)?;
            }

            zip.finish()
                .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
        }

        temp_file
            .persist(destination)
            .map_err(|e| DhapError::Io(e.error))?;

        Ok(())
    }

    /// Load a project and its frames from a .dhap ZIP archive.
    /// Protects against directory traversal and checks schema compatibility.
    pub fn load(source: &Path) -> DhapResult<(Project, HashMap<String, Vec<u8>>)> {
        if !source.exists() {
            return Err(DhapError::ProjectNotFound(
                source.to_string_lossy().to_string(),
            ));
        }

        let file = File::open(source)?;
        let mut archive = ZipArchive::new(BufReader::new(file))
            .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;

        // Path traversal defense check
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
            let name = entry.name();
            Self::validate_relative_path(name)?;
        }

        // 1. Read manifest.json
        let manifest: Manifest = {
            let mut manifest_file = archive
                .by_name("manifest.json")
                .map_err(|_| DhapError::CorruptArchive("manifest.json missing".to_string()))?;
            let mut contents = Vec::new();
            manifest_file.read_to_end(&mut contents)?;
            serde_json::from_slice(&contents)?
        };

        if manifest.schema_version > CURRENT_SCHEMA_VERSION {
            return Err(DhapError::InvalidSchemaVersion {
                expected: CURRENT_SCHEMA_VERSION,
                found: manifest.schema_version,
            });
        }

        // 2. Read project.json
        let project: Project = {
            let mut proj_file = archive
                .by_name("project.json")
                .map_err(|_| DhapError::CorruptArchive("project.json missing".to_string()))?;
            let mut contents = Vec::new();
            proj_file.read_to_end(&mut contents)?;
            serde_json::from_slice(&contents)?
        };

        // 3. Read frames and other assets
        let mut frames = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| DhapError::CorruptArchive(e.to_string()))?;
            let name = file.name().to_string();
            if name != "manifest.json" && name != "project.json" && !file.is_dir() {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                frames.insert(name, data);
            }
        }

        Ok((project, frames))
    }

    /// Strict path traversal validator
    pub fn validate_relative_path(path_str: &str) -> DhapResult<()> {
        if path_str.contains("..")
            || path_str.starts_with('/')
            || path_str.starts_with('\\')
            || path_str.contains(':')
        {
            return Err(DhapError::PathTraversal(path_str.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_bundle_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_path = temp_dir.path().join("sample_guide.dhap");

        let mut project = Project::new("E2E Test Guide");
        project.add_step("First Step", "Do something useful", None, Some(1000));

        let mut frames = HashMap::new();
        frames.insert(
            "frames/frame_001.png".to_string(),
            b"fake_png_binary_data".to_vec(),
        );

        ProjectBundle::save(&project, &frames, &target_path).expect("Save must succeed");
        assert!(target_path.exists());

        let (loaded_proj, loaded_frames) =
            ProjectBundle::load(&target_path).expect("Load must succeed");

        assert_eq!(loaded_proj.title, "E2E Test Guide");
        assert_eq!(loaded_proj.steps.len(), 1);
        assert_eq!(loaded_frames.len(), 1);
        assert_eq!(
            loaded_frames.get("frames/frame_001.png").unwrap(),
            &b"fake_png_binary_data".to_vec()
        );
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(ProjectBundle::validate_relative_path("../evil.txt").is_err());
        assert!(ProjectBundle::validate_relative_path("frames/../../etc/passwd").is_err());
        assert!(ProjectBundle::validate_relative_path("/root/file.txt").is_err());
        assert!(ProjectBundle::validate_relative_path("C:\\Windows\\System32").is_err());
        assert!(ProjectBundle::validate_relative_path("frames/step1.png").is_ok());
    }
}
