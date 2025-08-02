use crate::preflight::Manifest;
use std::fs::{create_dir_all, remove_dir_all, remove_file};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum AyarlaStatus {
    Ok,
    Warn,
}

pub fn lets_go(
    base_path: PathBuf,
    settings_dir_path: PathBuf,
    manifest: Manifest,
) -> Result<AyarlaStatus, anyhow::Error> {
    let mut status = AyarlaStatus::Ok;
    for item in manifest.manifest_items {
        let source_path = settings_dir_path.join(item.source);
        if !source_path.exists() {
            status = AyarlaStatus::Warn;
            continue;
        }

        let destination_path = base_path.join(item.destination);
        if destination_path.exists() {
            if item.force {
                if destination_path.is_dir() {
                    remove_dir_all(&destination_path)?;
                } else {
                    remove_file(&destination_path)?;
                }
            } else {
                continue;
            }
        }
        let parent = destination_path.parent().unwrap();
        if !parent.exists() {
            create_dir_all(parent)?;
        }

        let original = source_path.canonicalize()?;
        let link = destination_path;
        symlink(original, link)?;
    }
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preflight::ManifestItem;
    use std::fs::{self, DirEntry, File, create_dir_all};
    use tempfile::tempdir;

    fn get_test_manifest() -> Manifest {
        Manifest {
            manifest_items: vec![
                ManifestItem {
                    source: ".tmux.conf".to_string(),
                    destination: ".tmux.conf".to_string(),
                    force: false,
                },
                ManifestItem {
                    source: "nvim".to_string(),
                    destination: ".config/nvim".to_string(),
                    force: false,
                },
            ],
        }
    }

    #[test]
    fn lets_go_manifest_item_do_not_exist_nothing_happens_and_status_warn() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let settings_dir_path = temp_dir.path().join("settings_dir");
        create_dir_all(&settings_dir_path).expect("to create settings_dir");
        let home_dir_path = temp_dir.path().join("home");
        create_dir_all(&home_dir_path).expect("to create home dir");

        let result = lets_go(
            home_dir_path.to_path_buf(),
            settings_dir_path.to_path_buf(),
            get_test_manifest(),
        );

        assert_eq!(result.unwrap(), AyarlaStatus::Warn);
        assert_eq!(
            fs::read_dir(home_dir_path)
                .expect("to read home dir")
                .count(),
            0
        );
    }

    #[test]
    fn lets_go_manifest_one_item_exists_tmux_conf_configured_but_warn() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let settings_dir_path = temp_dir.path().join("settings_dir");
        let tmux_conf = settings_dir_path.join(".tmux.conf");
        create_dir_all(&settings_dir_path).expect("to create settings_dir");
        File::create(&tmux_conf).expect("to create .tmux.conf");
        let home_dir_path = temp_dir.path().join("home");
        create_dir_all(&home_dir_path).expect("to create home dir");

        let result = lets_go(
            home_dir_path.to_path_buf(),
            settings_dir_path.to_path_buf(),
            get_test_manifest(),
        );

        assert_eq!(result.unwrap(), AyarlaStatus::Warn);
        let just_as_expected = fs::read_dir(home_dir_path)
            .expect("to read settings_dir")
            .take(1)
            .map(|d| d.expect("to get dir entry"))
            .all(|d| d.file_name() == ".tmux.conf" && d.file_type().unwrap().is_symlink());
        assert!(just_as_expected);
    }

    #[test]
    fn lets_go_everything_configured_and_ok() {
        let temp_dir = tempdir().expect("to create temp_dir");
        let settings_dir_path = temp_dir.path().join("settings_dir");
        let tmux_conf = settings_dir_path.join(".tmux.conf");
        let nvim_dir_path = settings_dir_path.join("nvim");
        let nvim_conf_path = nvim_dir_path.join(".nvim");
        create_dir_all(&nvim_dir_path).expect("to create dir");
        File::create(&tmux_conf).expect("to create .tmux.conf");
        File::create(&nvim_conf_path).expect("to create .nvim file");
        let home_dir_path = temp_dir.path().join("home");
        create_dir_all(&home_dir_path).expect("to create home dir");

        let result = lets_go(
            home_dir_path.to_path_buf(),
            settings_dir_path.to_path_buf(),
            get_test_manifest(),
        );

        let dir_items = fs::read_dir(home_dir_path)
            .expect("to read settings_dir")
            .map(|d| d.expect("to get dir entry"))
            .collect::<Vec<DirEntry>>();
        assert_eq!(result.unwrap(), AyarlaStatus::Ok);
        assert!(dir_items.iter().any(|d| d.file_name() == ".tmux.conf"
            && d.file_type().expect("to get fileType").is_symlink()));
        let config_dir_in_home = dir_items
            .into_iter()
            .filter(|d| {
                d.file_name() == ".config" && d.file_type().expect("to get fileType").is_dir()
            })
            .collect::<Vec<DirEntry>>();
        assert!(config_dir_in_home.len() == 1);
        let config_dir_in_home = config_dir_in_home[0].path();
        let just_as_expected = fs::read_dir(config_dir_in_home)
            .expect("to read settings_dir")
            .take(1)
            .map(|d| d.expect("to get dir entry"))
            .all(|d| d.file_name() == "nvim" && d.file_type().unwrap().is_symlink());
        assert!(just_as_expected);
    }
}
