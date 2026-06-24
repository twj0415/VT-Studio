use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SafePath {
    absolute_path: PathBuf,
    relative_path: String,
}

#[derive(Debug, Clone)]
pub struct PathGuard {
    workspace_root: PathBuf,
}

impl SafePath {
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }

    pub fn relative_path(&self) -> &str {
        &self.relative_path
    }
}

impl PathGuard {
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }

    pub fn validate_relative_path(path: &str) -> Result<String, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("relative path cannot be empty.".to_string());
        }

        if trimmed != path {
            return Err("relative path cannot contain leading or trailing whitespace.".to_string());
        }

        if path.contains('\0') {
            return Err("relative path cannot contain null bytes.".to_string());
        }

        let lower = path.to_ascii_lowercase();
        if lower.starts_with("file://") {
            return Err("file:// paths are not allowed.".to_string());
        }

        if path.contains('\\') {
            return Err("relative path must use '/' separators.".to_string());
        }

        if path.starts_with('/') || path.starts_with("//") || path.contains(':') {
            return Err("relative path must not be absolute, UNC, or prefixed.".to_string());
        }

        if path.starts_with("./")
            || path.ends_with("/.")
            || path.contains("/./")
            || path.starts_with("../")
            || path.ends_with("/..")
            || path.contains("/../")
        {
            return Err(
                "relative path cannot contain current or parent directory segments.".to_string(),
            );
        }

        let mut normalized_segments = Vec::new();
        for segment in path.split('/') {
            if segment.is_empty() {
                return Err("relative path cannot contain empty segments.".to_string());
            }

            if segment == "." || segment == ".." {
                return Err(
                    "relative path cannot contain current or parent directory segments."
                        .to_string(),
                );
            }

            normalized_segments.push(segment);
        }

        let normalized = normalized_segments.join("/");
        let parsed = Path::new(&normalized);
        if parsed.is_absolute() {
            return Err("relative path must not be absolute.".to_string());
        }

        for component in parsed.components() {
            if !matches!(component, Component::Normal(_)) {
                return Err("relative path contains unsafe path components.".to_string());
            }
        }

        Ok(normalized)
    }

    pub fn safe_join_existing(&self, relative_path: &str) -> Result<SafePath, String> {
        let normalized = Self::validate_relative_path(relative_path)?;
        let canonical_workspace_root = self.canonical_workspace_root()?;
        let canonical_path =
            self.resolve_existing_without_reparse(&normalized, &canonical_workspace_root)?;

        Ok(SafePath {
            absolute_path: canonical_path,
            relative_path: normalized,
        })
    }

    pub fn safe_join_for_write(&self, relative_path: &str) -> Result<SafePath, String> {
        let normalized = Self::validate_relative_path(relative_path)?;
        fs::create_dir_all(&self.workspace_root).map_err(|error| error.to_string())?;
        let canonical_workspace_root = self.canonical_workspace_root()?;
        let mut segments = normalized.split('/').collect::<Vec<_>>();
        let file_name = segments
            .pop()
            .ok_or_else(|| "target path must include a file name.".to_string())?;
        let mut current_dir = canonical_workspace_root.clone();

        for segment in segments {
            let next_dir = current_dir.join(segment);
            match fs::symlink_metadata(&next_dir) {
                Ok(metadata) => {
                    ensure_not_link_or_reparse(&next_dir, &metadata)?;
                    if !metadata.is_dir() {
                        return Err("target parent path is not a directory.".to_string());
                    }

                    let canonical_next_dir =
                        next_dir.canonicalize().map_err(|error| error.to_string())?;
                    ensure_inside_workspace(&canonical_workspace_root, &canonical_next_dir)?;
                    current_dir = canonical_next_dir;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&next_dir).map_err(|error| error.to_string())?;
                    let canonical_next_dir =
                        next_dir.canonicalize().map_err(|error| error.to_string())?;
                    ensure_inside_workspace(&canonical_workspace_root, &canonical_next_dir)?;
                    current_dir = canonical_next_dir;
                }
                Err(error) => return Err(error.to_string()),
            }
        }

        let target = current_dir.join(file_name);
        let absolute_path = if let Ok(metadata) = fs::symlink_metadata(&target) {
            ensure_not_link_or_reparse(&target, &metadata)?;
            let canonical_target = target.canonicalize().map_err(|error| error.to_string())?;
            ensure_inside_workspace(&canonical_workspace_root, &canonical_target)?;
            canonical_target
        } else {
            target
        };

        Ok(SafePath {
            absolute_path,
            relative_path: normalized,
        })
    }

    #[allow(dead_code)]
    pub fn safe_zip_entry_for_write(&self, entry_path: &str) -> Result<SafePath, String> {
        self.safe_join_for_write(entry_path)
    }

    #[allow(dead_code)]
    pub fn safe_template_resource(&self, template_relative_path: &str) -> Result<SafePath, String> {
        let normalized = Self::validate_relative_path(template_relative_path)?;
        self.safe_join_existing(&format!("templates/{normalized}"))
    }

    #[allow(dead_code)]
    pub fn safe_ffmpeg_input(&self, relative_path: &str) -> Result<SafePath, String> {
        self.safe_join_existing(relative_path)
    }

    #[allow(dead_code)]
    pub fn safe_chromium_file(&self, relative_path: &str) -> Result<SafePath, String> {
        self.safe_join_existing(relative_path)
    }

    #[allow(dead_code)]
    pub fn safe_export_path_for_write(&self, relative_path: &str) -> Result<SafePath, String> {
        self.safe_join_for_write(relative_path)
    }

    #[allow(dead_code)]
    pub fn safe_import_target_for_write(&self, relative_path: &str) -> Result<SafePath, String> {
        self.safe_join_for_write(relative_path)
    }

    fn resolve_existing_without_reparse(
        &self,
        normalized: &str,
        canonical_workspace_root: &Path,
    ) -> Result<PathBuf, String> {
        let mut current = canonical_workspace_root.to_path_buf();
        for segment in normalized.split('/') {
            let next = current.join(segment);
            let metadata = fs::symlink_metadata(&next).map_err(|error| error.to_string())?;
            ensure_not_link_or_reparse(&next, &metadata)?;
            let canonical_next = next.canonicalize().map_err(|error| error.to_string())?;
            ensure_inside_workspace(canonical_workspace_root, &canonical_next)?;
            current = canonical_next;
        }

        Ok(current)
    }

    fn canonical_workspace_root(&self) -> Result<PathBuf, String> {
        let metadata =
            fs::symlink_metadata(&self.workspace_root).map_err(|error| error.to_string())?;
        ensure_not_link_or_reparse(&self.workspace_root, &metadata)?;
        self.workspace_root
            .canonicalize()
            .map_err(|error| error.to_string())
    }
}

fn ensure_inside_workspace(workspace_root: &Path, candidate: &Path) -> Result<(), String> {
    if candidate.starts_with(workspace_root) {
        return Ok(());
    }

    Err("resolved path is outside the controlled workspace.".to_string())
}

fn ensure_not_link_or_reparse(path: &Path, metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.file_type().is_symlink() || is_reparse_point(metadata) {
        return Err(format!(
            "path contains a symlink or reparse point: {}",
            path.display()
        ));
    }

    Ok(())
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
    use super::PathGuard;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_windows_and_url_path_bypasses() {
        for unsafe_path in [
            "../outside.txt",
            "..\\outside.txt",
            "project/../outside.txt",
            "project/./source.txt",
            "./project/source.txt",
            "C:/Windows/win.ini",
            "C:\\Windows\\win.ini",
            "//server/share/file.txt",
            "\\\\server\\share\\file.txt",
            "file:///C:/Windows/win.ini",
            "project//source.txt",
        ] {
            assert!(
                PathGuard::validate_relative_path(unsafe_path).is_err(),
                "{unsafe_path} should be rejected"
            );
        }
    }

    #[test]
    fn resolves_existing_files_only_inside_workspace() {
        let root = test_root("existing");
        fs::create_dir_all(root.join("projects/project_a/input"))
            .expect("project dir should exist");
        fs::write(root.join("projects/project_a/input/source.txt"), "ok")
            .expect("source should write");

        let guard = PathGuard::new(&root);
        let safe_path = guard
            .safe_join_existing("projects/project_a/input/source.txt")
            .expect("controlled file should resolve");

        assert_eq!(
            safe_path.relative_path(),
            "projects/project_a/input/source.txt"
        );
        assert!(safe_path.absolute_path().is_absolute());
        assert!(guard.safe_join_existing("../outside.txt").is_err());

        cleanup(root);
    }

    #[test]
    fn write_paths_are_resolved_after_parent_canonicalization() {
        let root = test_root("write");
        let guard = PathGuard::new(&root);

        let safe_path = guard
            .safe_join_for_write("projects/project_a/input/source.txt")
            .expect("controlled write path should resolve");
        fs::write(safe_path.absolute_path(), "ok").expect("file should write");

        let stored = fs::read_to_string(root.join("projects/project_a/input/source.txt"))
            .expect("stored file should read");
        assert_eq!(stored, "ok");
        assert!(guard.safe_join_for_write("../outside.txt").is_err());

        cleanup(root);
    }

    #[test]
    fn rejects_zip_slip_entries() {
        let root = test_root("zip");
        let guard = PathGuard::new(&root);

        assert!(guard.safe_zip_entry_for_write("../escape.txt").is_err());
        assert!(guard
            .safe_zip_entry_for_write("projects/project_a/assets/image.png")
            .is_ok());

        cleanup(root);
    }

    #[test]
    fn template_ffmpeg_and_chromium_paths_return_safe_paths() {
        let root = test_root("contexts");
        fs::create_dir_all(root.join("templates/default")).expect("template dir should exist");
        fs::create_dir_all(root.join("outputs/project_a")).expect("output dir should exist");
        fs::write(root.join("templates/default/preset.json"), "{}").expect("template should write");
        fs::write(root.join("outputs/project_a/scene.png"), "png").expect("output should write");

        let guard = PathGuard::new(&root);
        assert_eq!(
            guard
                .safe_template_resource("default/preset.json")
                .expect("template should resolve")
                .relative_path(),
            "templates/default/preset.json"
        );
        assert!(guard
            .safe_ffmpeg_input("outputs/project_a/scene.png")
            .is_ok());
        assert!(guard
            .safe_chromium_file("outputs/project_a/scene.png")
            .is_ok());
        assert!(guard
            .safe_chromium_file("file:///C:/Windows/win.ini")
            .is_err());

        cleanup(root);
    }

    #[test]
    fn rejects_existing_file_symlink_that_resolves_outside_workspace() {
        let root = test_root("file_link");
        let outside = test_root("file_link_outside");
        fs::create_dir_all(root.join("projects")).expect("project dir should exist");
        fs::create_dir_all(&outside).expect("outside dir should exist");
        let outside_file = outside.join("secret.txt");
        fs::write(&outside_file, "secret").expect("outside file should write");
        let link = root.join("projects/link.txt");

        if create_file_symlink(&outside_file, &link).is_err() {
            cleanup(root);
            cleanup(outside);
            return;
        }

        let guard = PathGuard::new(&root);
        assert!(guard.safe_join_existing("projects/link.txt").is_err());
        assert!(guard.safe_join_for_write("projects/link.txt").is_err());

        cleanup(root);
        cleanup(outside);
    }

    #[test]
    fn rejects_directory_symlink_parent_that_resolves_outside_workspace() {
        let root = test_root("dir_link");
        let outside = test_root("dir_link_outside");
        fs::create_dir_all(root.join("projects")).expect("project dir should exist");
        fs::create_dir_all(&outside).expect("outside dir should exist");
        let link = root.join("projects/outside_link");

        if create_dir_symlink(&outside, &link).is_err() {
            cleanup(root);
            cleanup(outside);
            return;
        }

        let guard = PathGuard::new(&root);
        assert!(guard
            .safe_join_for_write("projects/outside_link/new.txt")
            .is_err());

        cleanup(root);
        cleanup(outside);
    }

    fn test_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "vt-ai-short-video-maker-path-guard-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn cleanup(path: PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    #[cfg(unix)]
    fn create_file_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, link)
    }

    #[cfg(unix)]
    fn create_dir_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(source, link)
    }

    #[cfg(windows)]
    fn create_file_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(source, link)
    }

    #[cfg(windows)]
    fn create_dir_symlink(source: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_dir(source, link)
    }
}
