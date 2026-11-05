//! Shadow tree mirroring and incremental header cache for emocxx.
//!
//! Transpiles discovered local headers into a build-local cache:
//! `<cache_dir>/include/...`
//! and provides the directory path to prepend with `-iquote`.

use crate::lexer::transpile_source;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Extensions considered as C/C++ header files.
const HEADER_EXTENSIONS: &[&str] = &["h", "hpp", "hxx", "hh", "inl", "tpp"];

pub struct ShadowCache {
    cache_root: PathBuf,
}

impl ShadowCache {
    /// Creates or connects to a shadow cache rooted at `cache_root`.
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    /// Returns the shadow include directory: `<cache_root>/include`
    pub fn include_dir(&self) -> PathBuf {
        self.cache_root.join("include")
    }

    /// Returns the shadow source directory: `<cache_root>/src`
    pub fn src_dir(&self) -> PathBuf {
        self.cache_root.join("src")
    }

    /// Mirrors and transpiles headers from user include directories and source parent directories.
    /// Returns the shadow include path to inject via `-iquote` and path mappings for depfile rewriting.
    pub fn mirror_headers(
        &self,
        include_dirs: &[PathBuf],
    ) -> std::io::Result<(PathBuf, Vec<(PathBuf, PathBuf)>)> {
        let shadow_inc = self.include_dir();
        fs::create_dir_all(&shadow_inc)?;

        let mut visited = HashSet::new();
        let mut mappings = Vec::new();

        for dir in include_dirs {
            if dir.is_dir() {
                self.mirror_dir_recursive(dir, dir, &shadow_inc, &mut visited, &mut mappings)?;
            }
        }

        Ok((shadow_inc, mappings))
    }

    /// Recursively scans `current_dir` and mirrors all headers to `shadow_inc`
    /// preserving relative paths from `base_dir`.
    fn mirror_dir_recursive(
        &self,
        base_dir: &Path,
        current_dir: &Path,
        shadow_inc: &Path,
        visited: &mut HashSet<PathBuf>,
        mappings: &mut Vec<(PathBuf, PathBuf)>,
    ) -> std::io::Result<()> {
        let entries = match fs::read_dir(current_dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Avoid recursing into .git, build directories, or our own cache
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with('.') || file_name == "build" || file_name == "target" {
                    continue;
                }
                self.mirror_dir_recursive(base_dir, &path, shadow_inc, visited, mappings)?;
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if HEADER_EXTENSIONS.contains(&ext.as_str()) {
                    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
                    if visited.insert(canonical) {
                        self.transpile_and_cache_header(base_dir, &path, shadow_inc, mappings)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Transpiles a single header file if it has changed since last cached.
    fn transpile_and_cache_header(
        &self,
        base_dir: &Path,
        header_path: &Path,
        shadow_inc: &Path,
        mappings: &mut Vec<(PathBuf, PathBuf)>,
    ) -> std::io::Result<()> {
        let content = fs::read_to_string(header_path)?;

        // Compute hash of the raw content
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        let rel_path = header_path
            .strip_prefix(base_dir)
            .unwrap_or_else(|_| header_path.file_name().map(Path::new).unwrap_or(header_path));

        let dest_path = shadow_inc.join(rel_path);
        let hash_path = dest_path.with_extension(format!(
            "{}.hash",
            dest_path.extension().and_then(|s| s.to_str()).unwrap_or("h")
        ));

        // Also compute emoji-mangled alias path for emojis in filenames
        let rel_str = rel_path.to_string_lossy();
        let mangled_rel_str = crate::mangler::mangle_emojis_in_identifier(&rel_str);
        let mangled_dest = shadow_inc.join(&mangled_rel_str);

        // Record mappings for depfile sanitization
        mappings.push((dest_path.clone(), header_path.to_path_buf()));
        if mangled_dest != dest_path {
            mappings.push((mangled_dest.clone(), header_path.to_path_buf()));
        }

        // Check if cached file is already up to date
        if dest_path.exists() && hash_path.exists() {
            if let Ok(cached_hash_str) = fs::read_to_string(&hash_path) {
                if cached_hash_str.trim() == hash.to_string() {
                    return Ok(());
                }
            }
        }

        // Transpile header with #line directive pointing to original path
        let transpiled = transpile_source(&content, Some(header_path));

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&dest_path, &transpiled)?;
        let _ = fs::write(&hash_path, hash.to_string());

        // Write mangled alias if filename contains emojis
        if mangled_dest != dest_path {
            if let Some(parent) = mangled_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::write(&mangled_dest, &transpiled);
        }

        Ok(())
    }

    /// Transpiles a source file into the shadow source cache.
    /// Returns the path to the transpiled source file.
    pub fn transpile_source_file(&self, source_path: &Path) -> std::io::Result<PathBuf> {
        let shadow_src = self.src_dir();
        fs::create_dir_all(&shadow_src)?;

        let content = fs::read_to_string(source_path)?;

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        let raw_stem = source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("source");
        // Mangle emojis in filename stem to ensure host compiler receives pure ASCII paths
        let file_stem = crate::mangler::mangle_emojis_in_identifier(raw_stem);

        let ext = source_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("cpp");

        let dest_filename = format!("{}_{:x}.{}", file_stem, hash, ext);
        let dest_path = shadow_src.join(&dest_filename);

        if !dest_path.exists() {
            let transpiled = transpile_source(&content, Some(source_path));
            fs::write(&dest_path, transpiled)?;
        }

        Ok(dest_path)
    }
}

