use std::{path::{Path, PathBuf}, borrow::Cow, ffi::OsStr};

use crate::config::FLAKE_DIR;

pub trait PathExt {
    fn join_ignore_abs(&self, p: impl AsRef<Path>) -> PathBuf;
    fn with_root(&self, root: Option<impl AsRef<Path>>) -> RootedPath;
}

impl<T: AsRef<Path>> PathExt for T {
    fn join_ignore_abs(&self, p: impl AsRef<Path>) -> PathBuf {
        let p = p.as_ref();
        if p.is_absolute() {
            self.as_ref().join(p.components().skip(1).collect::<PathBuf>())
        } else {
            self.as_ref().join(p)
        }
    }

    fn with_root(&self, root: Option<impl AsRef<Path>>) -> RootedPath {
        RootedPath::from(self).with_root(root)
    }
}

pub fn flake_dir_from(root: Option<impl AsRef<Path>>) -> PathBuf {
    RootedPath {
        internal_path: FLAKE_DIR.to_owned(),
        root: root.map(|x| x.as_ref().to_owned())
    }.path_on_disk().to_path_buf()
}

/// A PathBuf that is either relative to the usual system root or to a fake root
pub struct RootedPath {
    internal_path: PathBuf,
    root: Option<PathBuf>
}

impl RootedPath {
    pub fn path(&self) -> &Path {
        &self.internal_path
    }

    pub fn path_on_disk(&self) -> Cow<'_, Path> {
        if let Some(ref root) = self.root {
            Cow::Owned(root.join_ignore_abs(&self.internal_path))
        } else {
            Cow::Borrowed(&self.internal_path)
        }
    }

    pub fn with_root(self, root: Option<impl AsRef<Path>>) -> Self {
        Self { internal_path: self.internal_path, root: root.map(|x| x.as_ref().to_owned()) }
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    pub fn has_fake_root(&self) -> bool {
        self.root.is_some()
    }

    pub fn file_name(&self) -> Option<&OsStr> {
        self.internal_path.file_name()
    }
}

impl<T: AsRef<Path>> From<T> for RootedPath {
    fn from(value: T) -> Self {
        Self { internal_path: value.as_ref().to_owned(), root: None }
    }
}