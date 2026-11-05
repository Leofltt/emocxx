//! Host compiler proxy and invocation engine for emocxx.
//!
//! Detects real C++ compiler (clang++, g++), parses arguments, handles non-compilation
//! passthroughs, manages shadow cache, and invokes the host compiler.

use crate::depfile::rewrite_depfile;
use crate::shadow::ShadowCache;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_EXTENSIONS: &[&str] = &["cpp", "cxx", "cc", "c++", "cp", "C"];

pub struct CompilerInvocation {
    pub real_cxx: PathBuf,
    pub original_args: Vec<String>,
    pub source_files: Vec<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub depfile: Option<PathBuf>,
    pub is_compile_only: bool,
    pub forwarded_args: Vec<String>,
}

impl CompilerInvocation {
    /// Parses CLI arguments and determines real compiler and operational mode.
    pub fn parse(raw_args: &[String]) -> Result<Self, String> {
        let mut real_cxx_override = None;
        let mut clean_args = Vec::new();

        let mut iter = raw_args.iter().peekable();
        while let Some(arg) = iter.next() {
            if let Some(val) = arg.strip_prefix("--real-cxx=") {
                real_cxx_override = Some(PathBuf::from(val));
            } else if arg == "--real-cxx" {
                if let Some(val) = iter.next() {
                    real_cxx_override = Some(PathBuf::from(val));
                }
            } else {
                clean_args.push(arg.clone());
            }
        }

        let real_cxx = resolve_real_compiler(real_cxx_override)?;

        let mut source_files = Vec::new();
        let mut include_dirs = Vec::new();
        let mut output_file = None;
        let mut depfile = None;
        let mut is_compile_only = false;
        let mut has_md = false;

        let mut i = 0;
        while i < clean_args.len() {
            let arg = &clean_args[i];

            if arg == "-c" {
                is_compile_only = true;
            } else if arg == "-o" && i + 1 < clean_args.len() {
                output_file = Some(PathBuf::from(&clean_args[i + 1]));
                i += 1;
            } else if let Some(val) = arg.strip_prefix("-o") {
                if !val.is_empty() {
                    output_file = Some(PathBuf::from(val));
                }
            } else if (arg == "-I" || arg == "-iquote" || arg == "-isystem") && i + 1 < clean_args.len() {
                include_dirs.push(PathBuf::from(&clean_args[i + 1]));
                i += 1;
            } else if let Some(val) = arg.strip_prefix("-I") {
                if !val.is_empty() {
                    include_dirs.push(PathBuf::from(val));
                }
            } else if let Some(val) = arg.strip_prefix("-iquote") {
                if !val.is_empty() {
                    include_dirs.push(PathBuf::from(val));
                }
            } else if let Some(val) = arg.strip_prefix("-isystem") {
                if !val.is_empty() {
                    include_dirs.push(PathBuf::from(val));
                }
            } else if arg == "-MF" && i + 1 < clean_args.len() {
                depfile = Some(PathBuf::from(&clean_args[i + 1]));
                i += 1;
            } else if let Some(val) = arg.strip_prefix("-MF") {
                if !val.is_empty() {
                    depfile = Some(PathBuf::from(val));
                }
            } else if arg == "-MD" || arg == "-MMD" {
                has_md = true;
            } else if !arg.starts_with('-') {
                let path = Path::new(arg);
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if SOURCE_EXTENSIONS.contains(&ext) && path.exists() {
                        source_files.push(path.to_path_buf());
                    }
                }
            }

            i += 1;
        }

        // If -MD is specified without explicit -MF, GCC/Clang writes to <out_stem>.d
        if has_md && depfile.is_none() {
            if let Some(ref out) = output_file {
                depfile = Some(out.with_extension("d"));
            } else if let Some(first_src) = source_files.first() {
                depfile = Some(first_src.with_extension("d"));
            }
        }

        Ok(Self {
            real_cxx,
            original_args: raw_args.to_vec(),
            source_files,
            include_dirs,
            output_file,
            depfile,
            is_compile_only,
            forwarded_args: clean_args,
        })
    }

    /// Determines cache root directory.
    pub fn cache_dir(&self) -> PathBuf {
        if let Ok(dir) = env::var("EMOCXX_CACHE_DIR") {
            return PathBuf::from(dir);
        }

        if let Some(ref out) = self.output_file {
            if let Some(parent) = out.parent() {
                // If output is inside build/, use build/.emocxx_cache
                let mut curr = parent;
                while let Some(p) = curr.parent() {
                    if curr.file_name().and_then(|s| s.to_str()) == Some("build") {
                        return curr.join(".emocxx_cache");
                    }
                    curr = p;
                }
                return parent.join(".emocxx_cache");
            }
        }

        PathBuf::from(".emocxx_cache")
    }

    /// Executes the compilation or transparent passthrough.
    pub fn execute(&self) -> Result<i32, String> {
        // Passthrough if no source files are being compiled (e.g. link steps, feature checks)
        if self.source_files.is_empty() {
            return self.run_passthrough();
        }

        let cache = ShadowCache::new(self.cache_dir());

        // Collect all include directories including source file parents
        let mut all_includes = self.include_dirs.clone();
        for src in &self.source_files {
            if let Some(parent) = src.parent() {
                if !all_includes.contains(&parent.to_path_buf()) {
                    all_includes.push(parent.to_path_buf());
                }
            }
        }

        // Mirror headers to shadow cache
        let shadow_inc = cache
            .mirror_headers(&all_includes)
            .map_err(|e| format!("Failed to mirror headers: {}", e))?;

        // Transpile sources to shadow cache
        let mut source_mappings = Vec::new();
        let mut final_args = Vec::new();

        // Inject shadow include at the very front
        final_args.push("-iquote".to_string());
        let shadow_inc_str = shadow_inc.canonicalize().unwrap_or(shadow_inc);
        final_args.push(shadow_inc_str.to_string_lossy().to_string());

        for arg in &self.forwarded_args {
            let path = PathBuf::from(arg);
            if self.source_files.contains(&path) {
                let transpiled = cache
                    .transpile_source_file(&path)
                    .map_err(|e| format!("Failed to transpile source {}: {}", path.display(), e))?;
                source_mappings.push((transpiled.clone(), path));
                final_args.push(transpiled.to_string_lossy().to_string());
            } else {
                final_args.push(arg.clone());
            }
        }

        // Run real compiler
        let status = Command::new(&self.real_cxx)
            .args(&final_args)
            .status()
            .map_err(|e| format!("Failed to invoke real compiler {}: {}", self.real_cxx.display(), e))?;

        let code = status.code().unwrap_or(1);
        if code == 0 {
            // Remap depfile paths if present
            if let Some(ref df) = self.depfile {
                let _ = rewrite_depfile(df, &source_mappings);
            }
        }

        Ok(code)
    }

    fn run_passthrough(&self) -> Result<i32, String> {
        let status = Command::new(&self.real_cxx)
            .args(&self.forwarded_args)
            .status()
            .map_err(|e| format!("Failed to execute passthrough {}: {}", self.real_cxx.display(), e))?;

        Ok(status.code().unwrap_or(1))
    }
}

/// Discovers the host C++ compiler, avoiding recursive self-invocation.
fn resolve_real_compiler(explicit: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("Specified --real-cxx does not exist: {}", path.display()));
    }

    if let Ok(env_cxx) = env::var("EMOCXX_REAL_CXX").or_else(|_| env::var("REAL_CXX")) {
        let p = PathBuf::from(env_cxx);
        if p.exists() {
            return Ok(p);
        }
    }

    let current_exe = env::current_exe().ok().and_then(|p| p.canonicalize().ok());

    // Search PATH for clang++ then g++
    let candidates = ["clang++", "g++"];
    for name in &candidates {
        if let Ok(path) = which::which(name) {
            let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
            if Some(&canon) != current_exe.as_ref() {
                return Ok(path);
            }
        }
    }

    // Default system fallbacks
    for path_str in &["/usr/bin/clang++", "/usr/bin/g++", "/usr/local/bin/clang++", "/opt/homebrew/bin/clang++"] {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Ok(p);
        }
    }

    Err("Could not detect host C++ compiler (clang++ or g++). Please set REAL_CXX or pass --real-cxx.".to_string())
}

mod which {
    use std::env;
    use std::path::PathBuf;

    pub fn which(binary_name: &str) -> Result<PathBuf, ()> {
        if let Ok(path_var) = env::var("PATH") {
            for dir in env::split_paths(&path_var) {
                let candidate = dir.join(binary_name);
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
        Err(())
    }
}

