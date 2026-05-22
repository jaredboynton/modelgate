use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use serde_json::Value;

use crate::{AppError, AppResult};

/// Parsed-JSON cache for small on-disk credential files (Codex `auth.json`,
/// `~/.ump/auth.json`, Cursor `auth.json`). Each provider auth helper
/// previously did `fs::read` + `serde_json::from_slice` on every request;
/// the cache collapses that to a `stat` plus a shared `Arc<Value>` walk
/// while metadata is unchanged.
///
/// Invariants:
/// * Cache state is keyed on the path, with `(modified, len)` carried per
///   entry. Any metadata change re-reads and re-parses.
/// * Absent files are cached as `None` so repeated probes stay at a single
///   `stat`.
/// * Parse errors are surfaced as `AppError::Json` and not cached, so a
///   subsequent fix is picked up immediately on the next request.
#[derive(Clone, Debug, Default)]
pub struct AuthFileCache {
    inner: Arc<Mutex<HashMap<PathBuf, CachedAuthFile>>>,
}

#[derive(Clone, Debug)]
struct CachedAuthFile {
    metadata: AuthFileMetadata,
    value: Option<Arc<Value>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuthFileMetadata {
    exists: bool,
    modified: Option<SystemTime>,
    len: u64,
}

impl AuthFileMetadata {
    fn absent() -> Self {
        Self {
            exists: false,
            modified: None,
            len: 0,
        }
    }
}

impl AuthFileCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_load(&self, path: &Path) -> AppResult<Option<Arc<Value>>> {
        let metadata = current_metadata(path)?;
        if let Some(cached) = self
            .inner
            .lock()
            .expect("auth file cache poisoned")
            .get(path)
            .filter(|cached| cached.metadata == metadata)
        {
            return Ok(cached.value.clone());
        }

        if !metadata.exists {
            self.inner.lock().expect("auth file cache poisoned").insert(
                path.to_path_buf(),
                CachedAuthFile {
                    metadata,
                    value: None,
                },
            );
            return Ok(None);
        }

        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let metadata = AuthFileMetadata::absent();
                self.inner.lock().expect("auth file cache poisoned").insert(
                    path.to_path_buf(),
                    CachedAuthFile {
                        metadata,
                        value: None,
                    },
                );
                return Ok(None);
            }
            Err(err) => return Err(AppError::Io(err)),
        };
        let value: Value = serde_json::from_slice(&bytes).map_err(AppError::Json)?;
        let value = Arc::new(value);
        self.inner.lock().expect("auth file cache poisoned").insert(
            path.to_path_buf(),
            CachedAuthFile {
                metadata,
                value: Some(Arc::clone(&value)),
            },
        );
        Ok(Some(value))
    }

    pub fn invalidate(&self, path: &Path) {
        self.inner
            .lock()
            .expect("auth file cache poisoned")
            .remove(path);
    }

    pub(crate) fn metadata(&self, path: &Path) -> AppResult<AuthFileMetadata> {
        current_metadata(path)
    }
}

pub(crate) fn current_metadata(path: &Path) -> AppResult<AuthFileMetadata> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(AuthFileMetadata {
            exists: true,
            modified: metadata.modified().ok(),
            len: metadata.len(),
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AuthFileMetadata::absent()),
        Err(err) => Err(AppError::Io(err)),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthCacheKey {
    parts: Vec<AuthCachePart>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AuthCachePart {
    File(PathBuf, AuthFileMetadata),
    Env(&'static str, Option<String>),
    Value(&'static str, Option<String>),
}

impl AuthCacheKey {
    pub(crate) fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub(crate) fn file(mut self, path: impl Into<PathBuf>) -> AppResult<Self> {
        let path = path.into();
        let metadata = current_metadata(&path)?;
        self.parts.push(AuthCachePart::File(path, metadata));
        Ok(self)
    }

    pub(crate) fn env(mut self, name: &'static str) -> Self {
        self.parts
            .push(AuthCachePart::Env(name, env::var(name).ok()));
        self
    }

    pub(crate) fn value(mut self, name: &'static str, value: Option<impl AsRef<OsStr>>) -> Self {
        let value = value.map(|value| value.as_ref().to_string_lossy().into_owned());
        self.parts.push(AuthCachePart::Value(name, value));
        self
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedAuthCache<T> {
    inner: Arc<Mutex<Option<ResolvedAuthEntry<T>>>>,
}

#[derive(Clone, Debug)]
struct ResolvedAuthEntry<T> {
    key: AuthCacheKey,
    value: AppResult<T>,
}

impl<T> Default for ResolvedAuthCache<T> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> ResolvedAuthCache<T>
where
    T: Clone,
{
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get_or_try_insert_with<F>(&self, key: AuthCacheKey, resolve: F) -> AppResult<T>
    where
        F: FnOnce() -> AppResult<T>,
    {
        if let Some(entry) = self
            .inner
            .lock()
            .expect("resolved auth cache poisoned")
            .as_ref()
            .filter(|entry| entry.key == key)
            .cloned()
        {
            return entry.value;
        }

        let value = resolve();
        *self.inner.lock().expect("resolved auth cache poisoned") = Some(ResolvedAuthEntry {
            key,
            value: value.clone(),
        });
        value
    }

    pub(crate) fn invalidate(&self) {
        *self.inner.lock().expect("resolved auth cache poisoned") = None;
    }
}
