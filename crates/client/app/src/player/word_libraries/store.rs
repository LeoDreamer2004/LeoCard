use crate::player::config_file;
use leocard_draw_guess::WordLibrary;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

#[derive(Clone)]
pub struct LocalWordLibrary {
    pub id: String,
    pub library: WordLibrary,
}

/// Independent JSON files leave the existing profile and preference archives untouched.
pub struct LocalWordLibraries {
    directory: Option<PathBuf>,
    pub entries: Vec<LocalWordLibrary>,
}

impl Default for LocalWordLibraries {
    fn default() -> Self {
        Self::at(config_file("draw-guess/word-libraries"))
    }
}

impl LocalWordLibraries {
    pub fn at(directory: Option<PathBuf>) -> Self {
        Self {
            directory,
            entries: Vec::new(),
        }
    }

    pub fn directory(&self) -> Result<&PathBuf, String> {
        self.directory
            .as_ref()
            .ok_or_else(|| "无法确定词库存储位置".to_owned())
    }

    pub fn reload(&mut self) -> Result<Vec<String>, String> {
        let directory = self.directory()?;
        let files = match fs::read_dir(directory) {
            Ok(files) => files,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.entries.clear();
                return Ok(Vec::new());
            }
            Err(error) => return Err(format!("无法读取词库文件夹：{error}")),
        };
        let mut entries = Vec::new();
        let mut warnings = Vec::new();
        for file in files {
            let result = (|| {
                let file = file.map_err(|error| error.to_string())?;
                if !file
                    .file_type()
                    .map_err(|error| error.to_string())?
                    .is_file()
                    || file
                        .path()
                        .extension()
                        .is_none_or(|extension| extension != "json")
                {
                    return Ok(());
                }
                let id = file.file_name().to_string_lossy().into_owned();
                if file.metadata().map_err(|error| error.to_string())?.len() > 524_288 {
                    return Err(format!("{id} 文件过大"));
                }
                let bytes = fs::read(file.path()).map_err(|error| format!("{id}：{error}"))?;
                let library: WordLibrary =
                    serde_json::from_slice(&bytes).map_err(|error| format!("{id}：{error}"))?;
                let library = library
                    .validate()
                    .map_err(|error| format!("{id}：{error}"))?;
                entries.push(LocalWordLibrary { id, library });
                Ok(())
            })();
            if let Err(error) = result {
                warnings.push(error);
            }
        }
        entries.sort_by(|a, b| a.library.title.cmp(&b.library.title));
        self.entries = entries;
        Ok(warnings)
    }

    pub fn save(&mut self, id: Option<&str>, library: WordLibrary) -> Result<String, String> {
        let library = library.validate().map_err(|error| error.to_string())?;
        if id.is_some_and(|id| !self.entries.iter().any(|entry| entry.id == id)) {
            return Err("词库不存在".to_owned());
        }
        let directory = self.directory()?;
        fs::create_dir_all(directory).map_err(|error| format!("无法创建词库文件夹：{error}"))?;
        let id = id
            .map(str::to_owned)
            .unwrap_or_else(|| format!("{:016x}.json", fastrand::u64(..)));
        let destination = directory.join(&id);
        let temporary = directory.join(format!(".save-{:016x}.tmp", fastrand::u64(..)));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            let bytes = serde_json::to_vec_pretty(&library)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, destination)
        })();
        if let Err(error) = result {
            let _ = fs::remove_file(temporary);
            return Err(format!("无法保存词库：{error}"));
        }
        self.entries.retain(|entry| entry.id != id);
        self.entries.push(LocalWordLibrary {
            id: id.clone(),
            library,
        });
        self.entries
            .sort_by(|a, b| a.library.title.cmp(&b.library.title));
        Ok(id)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        if !self.entries.iter().any(|entry| entry.id == id) {
            return Err("词库不存在".into());
        }
        fs::remove_file(self.directory()?.join(id))
            .map_err(|error| format!("无法删除词库：{error}"))?;
        self.entries.retain(|entry| entry.id != id);
        Ok(())
    }
}
