use emocxx::compiler::CompilerInvocation;
use emocxx::lexer;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("emocxx: error: no input files or arguments provided");
        eprintln!("Usage: emocxx [compiler options] <source-files>");
        eprintln!("       emocxx --transpile <file.cpp>");
        eprintln!("       emocxx --sanitize-compile-commands <compile_commands.json>");
        exit(1);
    }

    // Direct transpilation inspection utility
    if args[0] == "--transpile" {
        if args.len() < 2 {
            eprintln!("Usage: emocxx --transpile <file>");
            exit(1);
        }
        let path = Path::new(&args[1]);
        match fs::read_to_string(path) {
            Ok(content) => {
                let transpiled = lexer::transpile_source(&content, Some(path));
                print!("{}", transpiled);
                exit(0);
            }
            Err(e) => {
                eprintln!("emocxx: error reading {}: {}", path.display(), e);
                exit(1);
            }
        }
    }

    // compile_commands.json sanitizer utility (Phase 4)
    if args[0] == "--sanitize-compile-commands" {
        if args.len() < 2 {
            eprintln!("Usage: emocxx --sanitize-compile-commands <path-to-compile_commands.json>");
            exit(1);
        }
        let path = Path::new(&args[1]);
        if let Err(e) = sanitize_compile_commands(path) {
            eprintln!("emocxx: failed to sanitize compile_commands: {}", e);
            exit(1);
        }
        println!("Successfully sanitized {}", path.display());
        exit(0);
    }

    // Standard compiler proxy execution
    match CompilerInvocation::parse(&args) {
        Ok(invocation) => match invocation.execute() {
            Ok(code) => exit(code),
            Err(e) => {
                eprintln!("emocxx: internal error: {}", e);
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("emocxx: {}", e);
            exit(1);
        }
    }
}

/// Sanitizes a compile_commands.json file for language servers (clangd)
/// by ensuring paths to transpiled cache headers are visible and clean.
fn sanitize_compile_commands(path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    // Replace references to shadow cache with clean commands
    let sanitized = content.replace("--real-cxx=", "# real-cxx: ");
    fs::write(path, sanitized)?;
    Ok(())
}
