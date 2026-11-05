//! Dependency file (.d / -MF) remapping for build systems (Ninja, Make).
//!
//! Replaces shadow cache paths in generated `.d` files with original source and header paths.

use std::fs;
use std::path::{Path, PathBuf};

/// Rewrites a generated dependency file so that any references to shadow paths
/// are mapped back to their original user paths.
pub fn rewrite_depfile(
    depfile_path: &Path,
    mappings: &[(PathBuf, PathBuf)], // (shadow_path, original_path)
) -> std::io::Result<()> {
    if !depfile_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(depfile_path)?;
    let mut rewritten = content;

    for (shadow, original) in mappings {
        let shadow_str = shadow.to_string_lossy();
        let orig_str = original.to_string_lossy();

        rewritten = rewritten.replace(&*shadow_str, &*orig_str);

        // Also handle forward-slash variations on Windows/Mac
        let shadow_fwd = shadow_str.replace('\\', "/");
        let orig_fwd = orig_str.replace('\\', "/");
        if shadow_fwd != shadow_str {
            rewritten = rewritten.replace(&shadow_fwd, &orig_fwd);
        }
    }

    fs::write(depfile_path, rewritten)?;
    Ok(())
}

