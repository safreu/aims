use std::path::{Path, PathBuf};

pub(crate) struct PreparedBinary {
    _temp_dir: tempfile::TempDir,
    path: PathBuf,
}

impl PreparedBinary {
    pub(crate) fn new(temp_dir: tempfile::TempDir, path: PathBuf) -> Self {
        Self {
            _temp_dir: temp_dir,
            path,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    pub(crate) fn for_test(path: PathBuf) -> Self {
        let temp_dir = tempfile::tempdir().unwrap();

        Self {
            _temp_dir: temp_dir,
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn prepared_binary_exposes_binary_path() {
        let binary = PreparedBinary::for_test(PathBuf::from("/tmp/aimsctl-test"));

        assert_eq!(binary.path(), Path::new("/tmp/aimsctl-test"));
    }
}
