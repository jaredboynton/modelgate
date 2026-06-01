use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct RepoSandbox {
    root: PathBuf,
}

impl RepoSandbox {
    pub fn new(root: impl AsRef<Path>) -> AppResult<Self> {
        let root = fs::canonicalize(root.as_ref()).map_err(|error| {
            AppError::BadRequest(format!(
                "repo_root is not accessible ({}): {error}",
                root.as_ref().display()
            ))
        })?;
        if !root.is_dir() {
            return Err(AppError::BadRequest(format!(
                "repo_root must be a directory: {}",
                root.display()
            )));
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve_existing(&self, requested: impl AsRef<Path>) -> AppResult<PathBuf> {
        let requested = requested.as_ref();
        let joined = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            self.root.join(requested)
        };
        let resolved = fs::canonicalize(&joined).map_err(|error| {
            AppError::BadRequest(format!(
                "path is not accessible ({}): {error}",
                joined.display()
            ))
        })?;
        if !resolved.starts_with(&self.root) {
            return Err(AppError::BadRequest(format!(
                "path escapes repo_root: {}",
                requested.display()
            )));
        }
        Ok(resolved)
    }

    pub fn relative_display(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .trim_start_matches('/')
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_escape() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = RepoSandbox::new(temp.path()).unwrap();

        let error = sandbox.resolve_existing("../").unwrap_err();

        assert!(error.to_string().contains("escapes repo_root"));
    }

    #[test]
    fn rejects_symlink_escape() {
        let repo = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();
        std::os::unix::fs::symlink(&outside_file, repo.path().join("link.txt")).unwrap();
        let sandbox = RepoSandbox::new(repo.path()).unwrap();

        let error = sandbox.resolve_existing("link.txt").unwrap_err();

        assert!(error.to_string().contains("escapes repo_root"));
    }
}
