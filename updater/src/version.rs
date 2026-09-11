use semver::Version as SemVer;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(SemVer);

#[derive(Debug, Error)]
pub enum VersionParseError {
    #[error("invalid semantic version")]
    InvalidSemVer(semver::Error),
    #[error("prerelease versions are not allowed")]
    PrereleaseNotAllowed,
    #[error("build metadata is not allowed")]
    BuildMetadataNotAllowed,
}

impl Version {
    pub fn parse(input: &str) -> Result<Self, VersionParseError> {
        let version = SemVer::parse(input).map_err(VersionParseError::InvalidSemVer)?;

        if !version.pre.is_empty() {
            return Err(VersionParseError::PrereleaseNotAllowed);
        }

        if !version.build.is_empty() {
            return Err(VersionParseError::BuildMetadataNotAllowed);
        }

        Ok(Self(version))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_version_can_be_parsed() {
        let version = Version::parse("0.2.0");
        assert!(version.is_ok());
    }

    #[test]
    fn invalid_version_is_rejected() {
        let version = Version::parse("abc");
        assert!(version.is_err());
    }

    #[test]
    fn prerelease_version_is_rejected() {
        let version = Version::parse("1.0.0-beta.1");

        assert!(matches!(
            version,
            Err(VersionParseError::PrereleaseNotAllowed)
        ))
    }

    #[test]
    fn build_metadata_is_rejected() {
        let version = Version::parse("1.0.0+build7");

        assert!(matches!(
            version,
            Err(VersionParseError::BuildMetadataNotAllowed)
        ))
    }

    #[test]
    fn newer_version_is_greater() {
        let old = Version::parse("0.1.0").unwrap();
        let new = Version::parse("0.2.0").unwrap();

        assert!(new > old);
    }

    #[test]
    fn patch_version_is_greater() {
        let old = Version::parse("1.2.3").unwrap();
        let new = Version::parse("1.2.4").unwrap();

        assert!(new > old);
    }
}
