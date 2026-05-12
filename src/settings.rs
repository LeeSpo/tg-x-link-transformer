use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result};

pub struct LinkSettingsStore {
    path: PathBuf,
    values: Mutex<HashMap<i64, bool>>,
}

impl LinkSettingsStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let values = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).with_context(|| {
                format!("failed to parse link settings file {}", path.display())
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to read link settings file {}", path.display())
                })
            }
        };

        Ok(Self {
            path,
            values: Mutex::new(values),
        })
    }

    pub fn link_delete_enabled(&self, chat_id: i64) -> bool {
        self.values
            .lock()
            .expect("link settings mutex was poisoned")
            .get(&chat_id)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_link_delete(&self, chat_id: i64, enabled: bool) -> Result<()> {
        let updated_values = {
            let values = self
                .values
                .lock()
                .expect("link settings mutex was poisoned");
            let mut values = values.clone();
            values.insert(chat_id, enabled);
            values
        };

        self.persist(&updated_values)?;

        *self
            .values
            .lock()
            .expect("link settings mutex was poisoned") = updated_values;

        Ok(())
    }

    fn persist(&self, values: &HashMap<i64, bool>) -> Result<()> {
        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create link settings directory {}",
                    parent.display()
                )
            })?;
        }

        let file_name = self
            .path
            .file_name()
            .context("LINK_SETTINGS_PATH must include a file name")?
            .to_string_lossy();
        let temp_path = self.path.with_file_name(format!("{file_name}.tmp"));
        let contents =
            serde_json::to_string_pretty(values).context("failed to serialize link settings")?;

        fs::write(&temp_path, contents).with_context(|| {
            format!(
                "failed to write temporary link settings file {}",
                temp_path.display()
            )
        })?;
        fs::rename(&temp_path, &self.path).with_context(|| {
            format!(
                "failed to replace link settings file {} with {}",
                self.path.display(),
                temp_path.display()
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temp_settings_path(test_name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tg-x-link-transformer-{test_name}-{unique}.json"))
    }

    fn remove_if_exists(path: &Path) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("failed to remove {}: {error}", path.display()),
        }
    }

    #[test]
    fn missing_file_loads_empty_settings() {
        let path = temp_settings_path("missing-file");
        remove_if_exists(&path);

        let store = LinkSettingsStore::load(&path).unwrap();

        assert!(!store.link_delete_enabled(123));
    }

    #[test]
    fn set_true_persists_and_reloads() {
        let path = temp_settings_path("set-true");
        remove_if_exists(&path);

        let store = LinkSettingsStore::load(&path).unwrap();
        store.set_link_delete(123, true).unwrap();

        let reloaded = LinkSettingsStore::load(&path).unwrap();
        assert!(reloaded.link_delete_enabled(123));

        remove_if_exists(&path);
    }

    #[test]
    fn set_false_persists_and_reloads() {
        let path = temp_settings_path("set-false");
        remove_if_exists(&path);

        let store = LinkSettingsStore::load(&path).unwrap();
        store.set_link_delete(123, true).unwrap();
        store.set_link_delete(123, false).unwrap();

        let reloaded = LinkSettingsStore::load(&path).unwrap();
        assert!(!reloaded.link_delete_enabled(123));

        remove_if_exists(&path);
    }
}
