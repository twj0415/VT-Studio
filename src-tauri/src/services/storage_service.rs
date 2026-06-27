use crate::security::path_guard::PathGuard;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileBucket {
    Project,
    Asset,
    Output,
    Cache,
    Temp,
    Log,
    Template,
    Sidecar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FileAccessPolicy {
    ReadOnly,
    WriteProject,
    ExportCopy,
    TempOnly,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StoredFile {
    pub bucket: FileBucket,
    pub relative_path: String,
    pub absolute_path: PathBuf,
}

pub struct PathResolver {
    workspace_root: PathBuf,
    path_guard: PathGuard,
}

pub struct StorageService {
    resolver: PathResolver,
}

impl FileBucket {
    fn root_segment(self) -> &'static str {
        match self {
            Self::Project => "projects",
            Self::Asset => "assets",
            Self::Output => "outputs",
            Self::Cache => "cache",
            Self::Temp => "temp",
            Self::Log => "logs",
            Self::Template => "templates",
            Self::Sidecar => "sidecars",
        }
    }
}

impl PathResolver {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        let workspace_root = workspace_root.as_ref().to_path_buf();
        Self {
            path_guard: PathGuard::new(&workspace_root),
            workspace_root,
        }
    }

    #[allow(dead_code)]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn bucket_root(&self, bucket: FileBucket) -> PathBuf {
        self.workspace_root.join(bucket.root_segment())
    }

    pub fn canonical_bucket_root(&self, bucket: FileBucket) -> Result<PathBuf, String> {
        self.path_guard
            .safe_join_existing(bucket.root_segment())
            .map(|safe_path| safe_path.absolute_path().to_path_buf())
    }

    pub fn relative_bucket_path(
        &self,
        bucket: FileBucket,
        relative_path: &str,
    ) -> Result<String, String> {
        let normalized = PathGuard::validate_relative_path(relative_path)?;
        Ok(format!("{}/{}", bucket.root_segment(), normalized))
    }

    pub fn resolve_existing_bucket_path(
        &self,
        bucket: FileBucket,
        relative_path: &str,
    ) -> Result<PathBuf, String> {
        let bucket_path = self.relative_bucket_path(bucket, relative_path)?;
        let safe_path = self.path_guard.safe_join_existing(&bucket_path)?;
        let absolute_path = safe_path.absolute_path().to_path_buf();
        if safe_path.relative_path() != bucket_path {
            return Err("resolved path relative metadata changed unexpectedly.".to_string());
        }
        self.ensure_inside_bucket(bucket, &absolute_path)?;
        Ok(absolute_path)
    }

    pub fn resolve_bucket_path_for_write(
        &self,
        bucket: FileBucket,
        relative_path: &str,
    ) -> Result<PathBuf, String> {
        let bucket_path = self.relative_bucket_path(bucket, relative_path)?;
        let safe_path = self.path_guard.safe_join_for_write(&bucket_path)?;
        let absolute_path = safe_path.absolute_path().to_path_buf();
        if safe_path.relative_path() != bucket_path {
            return Err("resolved path relative metadata changed unexpectedly.".to_string());
        }
        self.ensure_inside_bucket(bucket, &absolute_path)?;
        Ok(absolute_path)
    }

    fn ensure_inside_bucket(&self, bucket: FileBucket, absolute_path: &Path) -> Result<(), String> {
        let bucket_root = self.canonical_bucket_root(bucket)?;
        if absolute_path.starts_with(&bucket_root) {
            return Ok(());
        }

        Err("resolved path is outside the requested storage bucket.".to_string())
    }
}

impl StorageService {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            resolver: PathResolver::new(workspace_root),
        }
    }

    pub fn initialize_workspace(&self) -> Result<(), String> {
        fs::create_dir_all(self.resolver.workspace_root()).map_err(|error| error.to_string())?;
        for bucket in [
            FileBucket::Project,
            FileBucket::Asset,
            FileBucket::Output,
            FileBucket::Cache,
            FileBucket::Temp,
            FileBucket::Log,
            FileBucket::Template,
            FileBucket::Sidecar,
        ] {
            let bucket_root = self.resolver.bucket_root(bucket);
            match fs::symlink_metadata(&bucket_root) {
                Ok(metadata) => {
                    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                        return Err(
                            "workspace bucket path cannot be a symlink or reparse point."
                                .to_string(),
                        );
                    }

                    if !metadata.is_dir() {
                        return Err("workspace bucket path is not a directory.".to_string());
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&bucket_root).map_err(|error| error.to_string())?;
                }
                Err(error) => return Err(error.to_string()),
            }

            self.resolver.canonical_bucket_root(bucket)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn resolver(&self) -> &PathResolver {
        &self.resolver
    }

    pub fn write_text(
        &self,
        bucket: FileBucket,
        relative_path: &str,
        content: &str,
        policy: FileAccessPolicy,
    ) -> Result<StoredFile, String> {
        ensure_write_allowed(bucket, policy)?;
        let absolute_path = self
            .resolver
            .resolve_bucket_path_for_write(bucket, relative_path)?;
        fs::write(&absolute_path, content).map_err(|error| error.to_string())?;
        self.to_stored_file(bucket, relative_path, absolute_path)
    }

    pub fn write_bytes(
        &self,
        bucket: FileBucket,
        relative_path: &str,
        content: &[u8],
        policy: FileAccessPolicy,
    ) -> Result<StoredFile, String> {
        ensure_write_allowed(bucket, policy)?;
        let absolute_path = self
            .resolver
            .resolve_bucket_path_for_write(bucket, relative_path)?;
        fs::write(&absolute_path, content).map_err(|error| error.to_string())?;
        self.to_stored_file(bucket, relative_path, absolute_path)
    }

    #[allow(dead_code)]
    pub fn copy_into_bucket(
        &self,
        source_path: &Path,
        bucket: FileBucket,
        relative_path: &str,
        policy: FileAccessPolicy,
    ) -> Result<StoredFile, String> {
        ensure_write_allowed(bucket, policy)?;
        if !source_path.is_file() {
            return Err("source_path must point to an existing file.".to_string());
        }

        let absolute_path = self
            .resolver
            .resolve_bucket_path_for_write(bucket, relative_path)?;
        fs::copy(source_path, &absolute_path).map_err(|error| error.to_string())?;
        self.to_stored_file(bucket, relative_path, absolute_path)
    }

    #[allow(dead_code)]
    pub fn read_to_string(
        &self,
        bucket: FileBucket,
        relative_path: &str,
        policy: FileAccessPolicy,
    ) -> Result<String, String> {
        ensure_read_allowed(policy)?;
        let absolute_path = self
            .resolver
            .resolve_existing_bucket_path(bucket, relative_path)?;
        fs::read_to_string(absolute_path).map_err(|error| error.to_string())
    }

    fn to_stored_file(
        &self,
        bucket: FileBucket,
        relative_path: &str,
        absolute_path: PathBuf,
    ) -> Result<StoredFile, String> {
        Ok(StoredFile {
            bucket,
            relative_path: self.resolver.relative_bucket_path(bucket, relative_path)?,
            absolute_path,
        })
    }
}

fn ensure_write_allowed(bucket: FileBucket, policy: FileAccessPolicy) -> Result<(), String> {
    match policy {
        FileAccessPolicy::ReadOnly => Err("read_only policy cannot write files.".to_string()),
        FileAccessPolicy::TempOnly if bucket != FileBucket::Temp => {
            Err("temp_only policy can only write to the temp bucket.".to_string())
        }
        _ => Ok(()),
    }
}

#[allow(dead_code)]
fn ensure_read_allowed(policy: FileAccessPolicy) -> Result<(), String> {
    match policy {
        FileAccessPolicy::ExportCopy | FileAccessPolicy::TempOnly => {
            Err("this file access policy does not allow preview reads.".to_string())
        }
        _ => Ok(()),
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{FileAccessPolicy, FileBucket, StorageService};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn initializes_all_workspace_buckets() {
        let root = test_root("init");
        let storage = StorageService::new(&root);
        storage
            .initialize_workspace()
            .expect("workspace should initialize");

        for segment in [
            "projects",
            "assets",
            "outputs",
            "cache",
            "temp",
            "logs",
            "templates",
            "sidecars",
        ] {
            assert!(root.join(segment).is_dir(), "{segment} should exist");
        }

        cleanup(root);
    }

    #[test]
    fn writes_and_reads_relative_project_file() {
        let root = test_root("write_read");
        let storage = StorageService::new(&root);
        storage
            .initialize_workspace()
            .expect("workspace should initialize");

        let stored = storage
            .write_text(
                FileBucket::Project,
                "project_a/input/source.txt",
                "hello",
                FileAccessPolicy::WriteProject,
            )
            .expect("file should write");

        assert_eq!(stored.relative_path, "projects/project_a/input/source.txt");
        assert_eq!(
            storage
                .read_to_string(
                    FileBucket::Project,
                    "project_a/input/source.txt",
                    FileAccessPolicy::ReadOnly,
                )
                .expect("file should read"),
            "hello"
        );

        cleanup(root);
    }

    #[test]
    fn copies_imported_asset_into_workspace() {
        let root = test_root("copy");
        let source_root = test_root("copy_source");
        fs::create_dir_all(&source_root).expect("source root should exist");
        let source = source_root.join("input.png");
        fs::write(&source, "png").expect("source file should write");
        let storage = StorageService::new(&root);
        storage
            .initialize_workspace()
            .expect("workspace should initialize");

        let stored = storage
            .copy_into_bucket(
                &source,
                FileBucket::Asset,
                "image/input.png",
                FileAccessPolicy::WriteProject,
            )
            .expect("asset should copy");

        assert_eq!(stored.relative_path, "assets/image/input.png");
        assert_eq!(
            fs::read_to_string(stored.absolute_path).expect("asset should read"),
            "png"
        );

        cleanup(root);
        cleanup(source_root);
    }

    #[test]
    fn rejects_unsafe_relative_paths() {
        let root = test_root("unsafe");
        let storage = StorageService::new(&root);

        assert!(storage
            .write_text(
                FileBucket::Project,
                "../outside.txt",
                "bad",
                FileAccessPolicy::WriteProject,
            )
            .is_err());
        assert!(storage
            .write_text(
                FileBucket::Project,
                "project/./source.txt",
                "bad",
                FileAccessPolicy::WriteProject,
            )
            .is_err());
        assert!(storage
            .write_text(
                FileBucket::Project,
                "C:/absolute.txt",
                "bad",
                FileAccessPolicy::WriteProject,
            )
            .is_err());

        cleanup(root);
    }

    #[test]
    fn temp_only_policy_cannot_write_outside_temp_bucket() {
        let root = test_root("temp_only");
        let storage = StorageService::new(&root);

        assert!(storage
            .write_text(
                FileBucket::Project,
                "project/source.txt",
                "bad",
                FileAccessPolicy::TempOnly,
            )
            .is_err());
        assert!(storage
            .write_text(
                FileBucket::Temp,
                "scratch/source.txt",
                "ok",
                FileAccessPolicy::TempOnly,
            )
            .is_ok());

        cleanup(root);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-storage-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn rejects_bucket_root_symlink_or_reparse_point() {
        let root = test_root("bucket_link");
        let outside = test_root("bucket_link_outside");
        fs::create_dir_all(&root).expect("workspace root should exist");
        fs::create_dir_all(&outside).expect("outside dir should exist");

        if create_dir_symlink(&outside, &root.join("assets")).is_err() {
            cleanup(root);
            cleanup(outside);
            return;
        }

        let storage = StorageService::new(&root);
        assert!(storage.initialize_workspace().is_err());

        cleanup(root);
        cleanup(outside);
    }

    #[cfg(unix)]
    fn create_dir_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, link)
    }

    #[cfg(windows)]
    fn create_dir_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(source, link)
    }
}
