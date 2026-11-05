# emocxx Benchmarking & Performance Suite

This directory contains the automated performance benchmarking harness and microbenchmarks for measuring the compilation overhead of the `emocxx` compiler proxy.

---

## Architecture & Fetch Strategy

Target C++ libraries are cloned on-demand as shallow git clones (`--depth 1 --branch <tag>`) directly into `.benchmarks/repos/` by the runner script.

- **In GitHub Actions**: The runner executes inside the Ubuntu container, shallow-clones the tagged library snapshots on-demand, runs the benchmarks, renders the summary table directly to the GitHub Step Summary, and uploads the report artifact.
- **Locally**: Developers can run the exact same command to reproduce the numbers or test optimizations on their local hardware.

---

## Local Reproduction Guide

### Prerequisites
- **Rust toolchain** (`cargo`, `rustc`)
- **Host C++ compiler** (`clang++` or `g++`)
- **CMake** ($\ge 3.20$) and **Ninja**
- **Python 3**
- **Git**

### 1. Build emocxx in Release Mode
```bash
cargo build --release
```

### 2. Run the Streaming Lexer Microbenchmark
Measures raw FSM lexing throughput (MB/s) on synthetic C++ files up to 100,000 lines with high emoji densities:

```bash
cargo run --release --bin bench_lexer
```

### 3. Run Library Benchmarks

#### Benchmark a Single Library (`fmt`):
```bash
python3 benchmarks/runner.py --library fmt --iterations 3
```

#### Benchmark All Configured Libraries:
```bash
python3 benchmarks/runner.py --library all --iterations 3
```

#### Custom Compiler or Job Count:
```bash
python3 benchmarks/runner.py --library spdlog --real-cxx /usr/bin/clang++ --jobs 8
```

Results will be printed to stdout and saved to `benchmarks/results/latest_report.md`.

---

## Supported Benchmark Libraries

| Library | Version | Characteristics | Purpose |
| :--- | :--- | :--- | :--- |
| **`fmt`** | 10.2.1 | Fast format strings, constexpr | Preprocessor and constexpr evaluation |
| **`spdlog`** | 1.13.0 | Multi-threaded logging | Deep include tree and header shadowing |
| **`json`** | 3.11.3 | Header-heavy modern C++ | Massive template metaprogramming |
| **`simdjson`** | 3.8.0 | Vectorized JSON parser | Modern C++20 standard compliance |
| **`boost`** | Latest | Asio, Beast, Spirit X3, MP11 | Extreme template depth, nested includes, and macro expansion |

---

## Adding a New Library

To add a new C++ project to the benchmark suite, add its entry to `LIBRARIES` in [`benchmarks/runner.py`](file:///Users/leofltt/repos/emocxx/benchmarks/runner.py):

```python
"my_lib": {
    "url": "https://github.com/user/my_lib.git",
    "tag": "v1.0.0",
    "cmake_args": ["-DBUILD_TESTING=OFF"],
    "touch_file": "src/main.cpp", # Optional: file to touch for incremental test
}
```
