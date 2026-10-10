use leocard_client::platform::install_apk;
use leocard_protocol::ProfileId;
use std::path::Path;

/// Android's package installer verifies signing and obtains user confirmation.
pub(crate) fn launch_installer(
    staged: &Path,
    _version: &str,
    _profile_id: ProfileId,
) -> Result<(), String> {
    install_apk(staged)
}
