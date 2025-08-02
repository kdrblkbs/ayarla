use anyhow::bail;
use serde::Deserialize;
use std::{
    fs::{DirEntry, read_dir, read_to_string},
    path::{Path, PathBuf},
};

const MANIFEST_FILE_NAME: &str = "manifest.toml";

#[derive(Deserialize)]
pub struct ManifestItem {
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Deserialize)]
pub struct Manifest {
    pub manifest_items: Vec<ManifestItem>,
}

pub fn checks(settings_directory: &str) -> Result<(PathBuf, Manifest), anyhow::Error> {
    let are_we_ready = are_we_ready_for_takeoff(settings_directory)?;
    let manifest = red_manifesto(are_we_ready.manifest_content)?;
    Ok((are_we_ready.settings_dir_path, manifest))
}

struct WeAreReadyMaybe {
    settings_dir_path: PathBuf,
    manifest_content: String,
}

fn are_we_ready_for_takeoff(settings_directory: &str) -> Result<WeAreReadyMaybe, anyhow::Error> {
    let settings_directory_path = Path::new(settings_directory);

    if !settings_directory_path.exists() {
        bail!("Directory does not exist: {}", settings_directory);
    }

    if !settings_directory_path.is_dir() {
        bail!("Path is not a directory: {}", settings_directory);
    }

    let dir_entries = read_dir(settings_directory_path)?
        .filter(|d| d.is_ok())
        .map(|d| d.unwrap())
        .collect::<Vec<DirEntry>>();

    if dir_entries.is_empty() {
        bail!("Directory is empty: {}", settings_directory);
    }

    if !dir_entries
        .iter()
        .any(|d| d.file_name().to_string_lossy() == MANIFEST_FILE_NAME)
    {
        bail!(
            "Directory does not contain manifest.toml: {}",
            settings_directory
        );
    }

    if dir_entries.len() == 1 {
        bail!(
            "Directory only contains manifest.toml: {}",
            settings_directory
        );
    }

    let manifest_content = read_to_string(settings_directory_path.join(MANIFEST_FILE_NAME))?;
    if manifest_content.is_empty() {
        bail!("manifest.toml in {} is empty", settings_directory);
    }

    Ok(WeAreReadyMaybe {
        settings_dir_path: settings_directory_path.to_path_buf(),
        manifest_content,
    })
}

///  TODO: fix the typos. it is supposed to be read_manifest ffs!
fn red_manifesto(manifest_content: String) -> Result<Manifest, anyhow::Error> {
    match toml::from_str(manifest_content.as_str()) {
        Ok(manifest) => Ok(manifest),
        Err(e) => bail!("Failed to parse manifest: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{File, create_dir_all},
        io::Write,
    };
    use tempfile::tempdir;

    #[test]
    fn are_we_ready_to_takeoff_with_not_existing_dir_assert_er() {
        let result = are_we_ready_for_takeoff("not_existing_directory");

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_settings_dir_not_a_dir_assert_err() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let some_file_path = temp_dir.path().join("some_file");
        File::create(&some_file_path).expect("to create some_file in temp_dir");

        let result =
            are_we_ready_for_takeoff(some_file_path.to_str().expect("to get str from path"));

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_empty_dir_assert_err() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("empty_directory");
        create_dir_all(&dir_path).expect("to create empty directory");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"));

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_no_manifest_assert_err() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("directory_with_a_file");
        create_dir_all(&dir_path).expect("to create a directory");
        let some_file_path = dir_path.join("not_a_manifest.json");
        File::create(&some_file_path).expect("to create a file");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"));

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_only_manifest_assert_err() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("directory_with_manifest");
        create_dir_all(&dir_path).expect("to create directory");
        let manifest = dir_path.join("manifest.toml");
        File::create(&manifest).expect("to create manifest.toml");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"));

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_empty_manifest_and_nvim_dir_assert_ok() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("settings_dir_with_nvim");
        let nvim_path = dir_path.join("nvim");
        create_dir_all(&nvim_path).expect("to create settings_dir_with_nvim");
        let manifest = dir_path.join("manifest.toml");
        File::create(&manifest).expect("to create manifest.toml");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"));

        assert!(result.is_err());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_manifest_and_nvim_dir_assert_ok() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("settings_dir_with_nvim");
        let nvim_path = dir_path.join("nvim");
        create_dir_all(&nvim_path).expect("to create settings_dir_with_nvim");
        let manifest = dir_path.join("manifest.toml");
        let mut manifest = File::create(&manifest).expect("to create manifest.toml");
        let manifest_content = "karl ftw!";
        manifest
            .write(manifest_content.as_bytes())
            .expect("to write to manifest");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"))
            .expect("to get result");

        assert_eq!(result.settings_dir_path, dir_path);
        assert_eq!(result.manifest_content, manifest_content.to_string());
    }

    #[test]
    fn are_we_ready_to_takeoff_with_manifest_and_tmux_conf_assert_ok() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("settings_dir_with_tmux_conf");
        create_dir_all(&dir_path).expect("to create settings_dir_with_nvim");
        let manifest = dir_path.join("manifest.toml");
        let mut manifest = File::create(&manifest).expect("to create manifest.toml");
        let manifest_content = "karl ftw!";
        manifest
            .write(manifest_content.as_bytes())
            .expect("to write to manifest");
        let tmux_conf = dir_path.join(".tmux.conf");
        File::create(&tmux_conf).expect("to create .tmux.conf");

        let result = are_we_ready_for_takeoff(dir_path.to_str().expect("to get str from path"))
            .expect("to get result");

        assert_eq!(result.settings_dir_path, dir_path);
        assert_eq!(result.manifest_content, manifest_content.to_string());
    }

    #[test]
    fn red_manifesto_with_empty_content_assert_err() {
        let result = red_manifesto(String::new());
        assert!(result.is_err())
    }

    #[test]
    fn red_manifesto_with_one_entry_assert_ok() {
        let content = r#"
[[manifest_items]]
source = "nvim"
destination = "./config/nvim"
force = true
"#;

        let manifest = red_manifesto(String::from(content)).expect("to get result");

        assert_eq!(manifest.manifest_items.len(), 1);
        assert_eq!(manifest.manifest_items[0].source, "nvim");
        assert_eq!(manifest.manifest_items[0].destination, "./config/nvim");
        assert_eq!(manifest.manifest_items[0].force, true);
    }

    #[test]
    fn red_manifesto_with_one_entry_but_different_order_assert_ok() {
        let content = r#"
[[manifest_items]]
destination = "./config/nvim"
source = "nvim"
force = true
"#;

        let manifest = red_manifesto(String::from(content)).expect("to get result");

        assert_eq!(manifest.manifest_items.len(), 1);
        assert_eq!(manifest.manifest_items[0].source, "nvim");
        assert_eq!(manifest.manifest_items[0].destination, "./config/nvim");
        assert_eq!(manifest.manifest_items[0].force, true);
    }

    #[test]
    fn red_manifesto_with_two_entries_assert_ok() {
        let content = r#"
[[manifest_items]]
source = "nvim"
destination = "./config/nvim"
force = true

[[manifest_items]]
source = "tmux.conf"
destination = ".tmux.conf"
force = false
"#;

        let manifest = red_manifesto(String::from(content)).expect("to get result");

        assert_eq!(manifest.manifest_items.len(), 2);
        assert_eq!(manifest.manifest_items[1].source, "tmux.conf");
        assert_eq!(manifest.manifest_items[1].destination, ".tmux.conf");
        assert_eq!(manifest.manifest_items[1].force, false);
    }

    #[test]
    fn red_manifesto_with_one_entry_and_default_values_assert_ok() {
        let content = r#"
[[manifest_items]]
source = "nvim"
destination = "./config/nvim"
"#;

        let manifest = red_manifesto(String::from(content)).expect("to get result");

        assert_eq!(manifest.manifest_items.len(), 1);
        assert_eq!(manifest.manifest_items[0].source, "nvim");
        assert_eq!(manifest.manifest_items[0].destination, "./config/nvim");
        assert_eq!(manifest.manifest_items[0].force, false);
    }

    #[test]
    fn red_manifesto_with_one_entry_but_missing_destination_assert_err() {
        let content = r#"
[[manifest_items]]
source = "nvim"
force = false
"#;

        let manifest = red_manifesto(String::from(content));

        assert!(manifest.is_err())
    }

    #[test]
    fn red_manifesto_with_one_entry_but_missing_source_assert_err() {
        let content = r#"
[[manifest_items]]
destination = "./config/nvim"
force = false
"#;

        let manifest = red_manifesto(String::from(content));

        assert!(manifest.is_err())
    }

    #[test]
    fn red_manifesto_with_two_entries_but_missing_required_fields_assert_err() {
        let content = r#"
[[manifest_items]]
destination = "./config/nvim"

[[manifest_items]]
source = "tmux.conf"
"#;

        let manifest = red_manifesto(String::from(content));

        assert!(manifest.is_err())
    }

    #[test]
    fn checks_happy_path_assert_ok() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let dir_path = temp_dir.path().join("settings_dir_with_tmux_conf");
        create_dir_all(&dir_path).expect("to create settings_dir_with_nvim");
        let manifest = dir_path.join("manifest.toml");
        let mut manifest = File::create(&manifest).expect("to create manifest.toml");
        let manifest_content = r#"
[[manifest_items]]
source = "tmux.conf"
destination = ".tmux.conf"
force = true
"#;
        manifest
            .write(manifest_content.as_bytes())
            .expect("to write to manifest");
        let tmux_conf = dir_path.join(".tmux.conf");
        File::create(&tmux_conf).expect("to create .tmux.conf");

        let result =
            checks(dir_path.to_str().expect("to get str from path")).expect("to get results");

        assert_eq!(result.0, dir_path);
        assert_eq!(result.1.manifest_items[0].source, "tmux.conf");
        assert_eq!(result.1.manifest_items[0].destination, ".tmux.conf");
        assert_eq!(result.1.manifest_items[0].force, true);
    }
}
