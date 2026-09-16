use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use sha2::{Digest, Sha256, Sha512};

use crate::installation::version::Version;

use super::prepared_binary::PreparedBinary;

pub(crate) trait TargetBinaryProvider {
    fn prepare(&self, version: &Version) -> Result<PreparedBinary, TargetBinaryProviderError>;
}

pub struct GithubReleaseBinaryProvider;

impl TargetBinaryProvider for GithubReleaseBinaryProvider {
    fn prepare(&self, version: &Version) -> Result<PreparedBinary, TargetBinaryProviderError> {
        let architecture = release_architecture()?;
        let asset_name = format!("aimsctl-linux-{architecture}");

        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("safreu")
            .repo_name("aims")
            .build()
            .map_err(TargetBinaryProviderError::ReleaseProvider)?
            .fetch()
            .map_err(TargetBinaryProviderError::ReleaseProvider)?;

        let release = releases
            .into_iter()
            .find(|release| release.version() == version.to_string())
            .ok_or_else(|| TargetBinaryProviderError::ReleaseNotFound(version.clone()))?;

        let asset = release
            .assets()
            .iter()
            .find(|asset| asset.name() == asset_name)
            .ok_or_else(|| TargetBinaryProviderError::ReleaseAssetNotFound {
                version: version.clone(),
                asset: asset_name.clone(),
            })?;

        let checksum_asset = release
            .assets()
            .iter()
            .find(|asset| asset.name() == "SHA256SUMS")
            .ok_or_else(|| TargetBinaryProviderError::ReleaseAssetNotFound {
                version: version.clone(),
                asset: "SHA256SUMS".to_string(),
            })?;

        let temp_dir =
            tempfile::tempdir().map_err(TargetBinaryProviderError::CreateTemporaryDirectory)?;

        let binary_path = temp_dir.path().join(&asset_name);
        let checksum_path = temp_dir.path().join("SHA256SUMS");

        let mut binary = fs::File::create(&binary_path)
            .map_err(TargetBinaryProviderError::CreateTargetBinary)?;

        let mut checksum_file = fs::File::create(&checksum_path)
            .map_err(TargetBinaryProviderError::CreateChecksumFile)?;

        self_update::Download::from_url(asset.download_url())
            .request_header(
                self_update::http::header::ACCEPT,
                "application/octet-stream",
            )
            .download_to(&mut binary)
            .map_err(TargetBinaryProviderError::DownloadTargetBinary)?;

        self_update::Download::from_url(checksum_asset.download_url())
            .request_header(
                self_update::http::header::ACCEPT,
                "application/octet-stream",
            )
            .download_to(&mut checksum_file)
            .map_err(TargetBinaryProviderError::DownloadChecksumFile)?;

        let sums = fs::read_to_string(&checksum_path)
            .map_err(TargetBinaryProviderError::ReadChecksumFile)?;

        let checksum = self_update::Checksum::from_sums_file(&sums, &asset_name)
            .map_err(TargetBinaryProviderError::InvalidChecksumFile)?;

        verify_checksum(&binary_path, &checksum)?;

        let mut permissions = fs::metadata(&binary_path)
            .map_err(TargetBinaryProviderError::ReadTargetBinaryMetadata)?
            .permissions();

        permissions.set_mode(0o755);

        fs::set_permissions(&binary_path, permissions)
            .map_err(TargetBinaryProviderError::SetTargetBinaryPermissions)?;

        Ok(PreparedBinary::new(temp_dir, binary_path))
    }
}

fn release_architecture() -> Result<&'static str, TargetBinaryProviderError> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        architecture => Err(TargetBinaryProviderError::UnsupportedArchitecture(
            architecture.to_string(),
        )),
    }
}

fn verify_checksum(
    path: &Path,
    expected: &self_update::Checksum,
) -> Result<(), TargetBinaryProviderError> {
    let contents = fs::read(path).map_err(TargetBinaryProviderError::ReadTargetBinary)?;

    let computed = match expected {
        self_update::Checksum::Sha256(_) => {
            let digest = Sha256::digest(&contents);

            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        }
        self_update::Checksum::Sha512(_) => {
            let digest = Sha512::digest(&contents);

            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        }
        _ => {
            return Err(TargetBinaryProviderError::UnsupportedChecksum);
        }
    };

    let expected = match expected {
        self_update::Checksum::Sha256(checksum) | self_update::Checksum::Sha512(checksum) => {
            checksum
        }
        _ => {
            return Err(TargetBinaryProviderError::UnsupportedChecksum);
        }
    };

    if !computed.eq_ignore_ascii_case(expected) {
        return Err(TargetBinaryProviderError::ChecksumMismatch);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TargetBinaryProviderError {
    #[error("unsupported architecture: {0}")]
    UnsupportedArchitecture(String),

    #[error("failed to access GitHub releases")]
    ReleaseProvider(#[source] self_update::errors::Error),

    #[error("Aims release {0} was not found")]
    ReleaseNotFound(Version),

    #[error("release {version} does not contain required asset {asset}")]
    ReleaseAssetNotFound { version: Version, asset: String },

    #[error("failed to create temporary directory")]
    CreateTemporaryDirectory(#[source] std::io::Error),

    #[error("failed to create temporary aimsctl binary")]
    CreateTargetBinary(#[source] std::io::Error),

    #[error("failed to download target aimsctl binary")]
    DownloadTargetBinary(#[source] self_update::errors::Error),

    #[error("failed to create temporary checksum file")]
    CreateChecksumFile(#[source] std::io::Error),

    #[error("failed to download release checksum file")]
    DownloadChecksumFile(#[source] self_update::errors::Error),

    #[error("failed to read release checksum file")]
    ReadChecksumFile(#[source] std::io::Error),

    #[error("release checksum file is invalid")]
    InvalidChecksumFile(#[source] self_update::errors::Error),

    #[error("failed to read downloaded aimsctl binary")]
    ReadTargetBinary(#[source] std::io::Error),

    #[error("unsupported checksum algorithm")]
    UnsupportedChecksum,

    #[error("downloaded aimsctl binary does not match its published checksum")]
    ChecksumMismatch,

    #[error("failed to read downloaded aimsctl metadata")]
    ReadTargetBinaryMetadata(#[source] std::io::Error),

    #[error("failed to make downloaded aimsctl executable")]
    SetTargetBinaryPermissions(#[source] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn temporary_file(contents: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("aimsctl-test");

        let mut file = fs::File::create(&path).unwrap();
        file.write_all(contents).unwrap();

        (temp_dir, path)
    }

    #[test]
    fn supported_architecture_is_mapped_to_release_architecture() {
        let architecture = release_architecture().unwrap();

        match std::env::consts::ARCH {
            "x86_64" => assert_eq!(architecture, "amd64"),
            "aarch64" => assert_eq!(architecture, "arm64"),
            architecture => {
                panic!("test running on unsupported architecture {architecture}")
            }
        }
    }

    #[test]
    fn valid_sha256_checksum_is_accepted() {
        let contents = b"aims test binary";
        let (_temp_dir, path) = temporary_file(contents);

        let digest = Sha256::digest(contents);

        let expected = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        let checksum = self_update::Checksum::Sha256(expected);

        let result = verify_checksum(&path, &checksum);

        assert!(result.is_ok());
    }

    #[test]
    fn invalid_sha256_checksum_is_rejected() {
        let (_temp_dir, path) = temporary_file(b"aims test binary");

        let checksum = self_update::Checksum::Sha256(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        );

        let result = verify_checksum(&path, &checksum);

        assert!(matches!(
            result,
            Err(TargetBinaryProviderError::ChecksumMismatch)
        ));
    }

    #[test]
    fn valid_sha512_checksum_is_accepted() {
        let contents = b"aims test binary";
        let (_temp_dir, path) = temporary_file(contents);

        let digest = Sha512::digest(contents);

        let expected = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        let checksum = self_update::Checksum::Sha512(expected);

        let result = verify_checksum(&path, &checksum);

        assert!(result.is_ok());
    }
}
