//! Micro-benchmark for emocxx streaming lexer throughput.
//!
//! Generates synthetic C++ files with heavy emoji density and measures:
//! - Raw throughput in Megabytes/second (MB/s)
//! - Number of emoji tokens mangled per second

use emocxx::lexer::transpile;
use std::time::Instant;

fn generate_synthetic_cpp(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 80);
    s.push_str("#pragma once\n#include <iostream>\n#include <vector>\n\n");

    for i in 0..lines {
        match i % 5 {
            0 => s.push_str(&format!("struct 📦_{} {{ int 💰_{}; int ⚖️_{}; }};\n", i, i, i)),
            1 => s.push_str(&format!("int 🚀_{} = {};\n", i, i)),
            2 => s.push_str(&format!(
                "auto 💥_{} = 📦_{}{{ .💰_{} = {}, .⚖️_{} = 10 }}.💰_{} * 🚀_{};\n",
                i,
                i - 2,
                i - 2,
                i,
                i - 2,
                i - 2,
                i - 1
            )),
            3 => s.push_str(&format!(
                "// Comment preserving 🚀 and 🎯 on line {}\nconst char* s_{} = \"Rocket: 🚀\";\n",
                i, i
            )),
            _ => s.push_str(&format!(
                "const char* raw_{} = R\"delim(Preserving 👨‍💻 and 🎯 verbatim)delim\";\n",
                i
            )),
        }
    }
    s
}

fn main() {
    let line_counts = [5_000, 25_000, 100_000];

    println!("=======================================================");
    println!("          emocxx Lexer Throughput Benchmark            ");
    println!("=======================================================\n");

    for &lines in &line_counts {
        let code = generate_synthetic_cpp(lines);
        let bytes = code.len();
        let mb = bytes as f64 / (1024.0 * 1024.0);

        // Warm up
        let _ = transpile(&code[..1000.min(code.len())]);

        let start = Instant::now();
        let transpiled = transpile(&code);
        let elapsed = start.elapsed().as_secs_f64();

        let throughput = mb / elapsed;

        println!(
            "[{:6} lines / {:5.2} MB] Processed in {:6.2} ms | Throughput: {:6.1} MB/s",
            lines,
            mb,
            elapsed * 1000.0,
            throughput
        );

        assert!(!transpiled.is_empty());
    }
    println!("\nThroughput benchmark completed.");
}

