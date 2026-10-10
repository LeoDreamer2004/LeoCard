use leocard_protocol::ProfileId;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

const RECEIPT_VERSION: u8 = 1;

#[derive(Deserialize, Serialize)]
pub(super) struct UpdateReceipt {
    profile_id: ProfileId,
    previous: String,
    installed: String,
}

impl UpdateReceipt {
    pub(super) fn stage(
        staged: &Path,
        installed: &str,
        profile_id: ProfileId,
    ) -> Result<PathBuf, String> {
        let receipt = Self {
            profile_id,
            previous: env!("CARGO_PKG_VERSION").to_owned(),
            installed: installed.to_owned(),
        };
        if !receipt.is_upgrade() {
            return Err("更新目标必须比当前版本更新".to_owned());
        }
        let mut bytes = vec![RECEIPT_VERSION];
        bytes.extend(
            postcard::to_allocvec(&receipt)
                .map_err(|error| format!("无法编码更新凭据：{error}"))?,
        );
        let path = sidecar(staged, ".receipt");
        fs::write(&path, bytes).map_err(|error| format!("无法保存更新凭据：{error}"))?;
        Ok(path)
    }

    /// This marker becomes visible only after the executable was successfully replaced.
    pub(super) fn complete(pending: &Path, target: &Path) -> io::Result<()> {
        fs::copy(pending, sidecar(target, ".update-completed"))?;
        let _ = fs::remove_file(pending);
        Ok(())
    }

    fn is_upgrade(&self) -> bool {
        match (
            Version::parse(&self.previous),
            Version::parse(&self.installed),
        ) {
            (Ok(previous), Ok(installed)) => installed > previous,
            _ => false,
        }
    }

    fn belongs_to(&self, profile_id: ProfileId, version: &str) -> bool {
        self.profile_id == profile_id && self.installed == version && self.is_upgrade()
    }
}

pub(crate) fn completed_update(profile_id: ProfileId) -> Result<bool, String> {
    let executable = env::current_exe().map_err(|error| format!("无法定位当前程序：{error}"))?;
    read_completed(&executable, profile_id, env!("CARGO_PKG_VERSION"))
}

fn read_completed(executable: &Path, profile_id: ProfileId, version: &str) -> Result<bool, String> {
    let bytes = match fs::read(sidecar(executable, ".update-completed")) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("无法读取更新凭据：{error}")),
    };
    if bytes.first() != Some(&RECEIPT_VERSION) {
        return Err("更新凭据格式版本不受支持".to_owned());
    }
    let receipt: UpdateReceipt =
        postcard::from_bytes(&bytes[1..]).map_err(|error| format!("无法解析更新凭据：{error}"))?;
    Ok(receipt.belongs_to(profile_id, version))
}

fn sidecar(executable: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(executable.as_os_str());
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
