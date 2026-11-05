# emocxx: Unicode Emoji Compiler Proxy for C++

`emocxx` is a transparent compiler proxy and source transpiler that allows arbitrary Unicode emojis to be used as identifiers (variables, structs, functions, member fields, templates) in C++ source trees (C++20/C++23) without modifying standard file extensions or breaking host build tooling (`clang++`, `g++`, CMake, Ninja).

---

## The Problem

The ISO C++23 standard (incorporating **P1949 / UAX #31**) strictly restricts valid identifier characters to Unicode properties `XID_Start` and `XID_Continue`. As a consequence, symbols and pictorial emojis are completely prohibited by native compilers, causing fatal lexical errors:

```text
error: unexpected character <U+1F680>
```

`emocxx` resolves this limitation without patching system compilers or writing a full C++ parser. By intercepting compile operations at the build-system boundary, it replaces emoji tokens with deterministic, collision-resistant ASCII identifiers, maintains continuous line-number mapping, and passes sanitized files directly to the host compiler.

---

## Features

- **Streaming FSM Lexer (UAX #29)**: Accurate grapheme segmentation handling single scalars, Variation Selector-16, compound skin-tone modifiers, and Zero-Width Joiner (ZWJ) sequences.
- **Literal Invariance**: String literals (`"Rocket: 🚀"`), raw string literals (`R"(🚀)"`), character literals (`'🚀'`), and comments (`// 🚀`, `/* 🚀 */`) remain untouched.
- **Shadow Header Mirroring**: Local header dependencies (`#include "types.hpp"`) containing emoji declarations are incrementally hashed and transpiled into a build-local cache (`<build>/.emocxx_cache/include/`).
- **Source Transparency**: Output files prepend `#line 1 "/path/to/source.cpp"`, preserving 1:1 line numbers in compilation errors and DWARF debugger symbols for **GDB** and **LLDB**.
- **Build System Depfile Sanitization**: Generated dependency files (`.d` / `-MF`) are automatically sanitized to preserve original source dependencies for Ninja and Make.
- **CMake & Ninja Ready**: Integrates as `CMAKE_CXX_COMPILER` or via the bundled `emocxx-toolchain.cmake`.

---

## Grapheme Mangling Specification

To comply with standard C++ identifier rules (`^[a-zA-Z_][a-zA-Z0-9_]*$`) and prevent namespace collisions, tokens are mangled using the schema:
$$\texttt{\_EMO\_<codepoints\_hex>\_}$$

| Emoji Input | Unicode Codepoints | Mangled C++ Token | Handling Mechanism |
| :--- | :--- | :--- | :--- |
| `🚀` | `U+1F680` | `_EMO_1f680_` | Single scalar symbol |
| `🏎️` | `U+1F3CE, U+FE0F` | `_EMO_1f3ce_fe0f_` | Variation Selector-16 preserved |
| `👍🏽` | `U+1F44D, U+1F3FD` | `_EMO_1f44d_1f3fd_` | Compound skin-tone modifier |
| `👨‍💻` | `U+1F468, U+200D, U+1F4BB` | `_EMO_1f468_200d_1f4bb_` | Zero-Width Joiner sequence |
| `"Rocket: 🚀"` | Literal sequence | `"Rocket: 🚀"` | String literal invariant |

---

## Quick Example

### Header (`include/engine.hpp`)
```cpp
#pragma once

struct 📦 {
    int 💰;
    int ⚖️;
};
```

### Source (`src/main.cpp`)
```cpp
#include <iostream>
#include "engine.hpp"

int main() {
    📦 🎁{ .💰 = 500, .⚖️ = 20 };
    int 🚀 = 2;
    auto 💥 = 🎁.💰 * 🚀;

    std::cout << "Calculated Score: " << 💥 << " 🎯\n";
    return (💥 == 1000) ? 0 : 1;
}
```

---

## Building emocxx

```bash
cargo build --release
```

The resulting binary will be in `target/release/emocxx`.

---

## CMake Integration

### Method 1: Using the Bundled Toolchain
```bash
cmake -B build -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE=cmake/emocxx-toolchain.cmake \
  -DREAL_CXX=clang++

cmake --build build
```

### Method 2: Direct Command Line Invocation
```bash
cmake -B build -G Ninja \
  -DCMAKE_CXX_COMPILER=/path/to/target/release/emocxx \
  -DREAL_CXX=clang++

cmake --build build
```

---

## Running Acceptance Tests & Debugger Validation

1. Run unit tests:
```bash
cargo test
```

2. Build and run the acceptance test:
```bash
cmake -B tests/acceptance/build -S tests/acceptance -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE=cmake/emocxx-toolchain.cmake \
  -DREAL_CXX=clang++

cmake --build tests/acceptance/build
./tests/acceptance/build/test_app
```

3. Verify LLDB breakpoint fidelity on line 9 of `main.cpp`:
```bash
lldb ./tests/acceptance/build/test_app \
  -o "breakpoint set --file main.cpp --line 9" \
  -o "run" \
  -o "quit"
```

---

## Benchmarking & Performance

An automated benchmark harness is provided in `benchmarks/` to measure compilation latency and compare baseline compiler performance against `emocxx`:

### 1. Library Benchmarking Suite
Tests real-world C++ libraries (`fmt`, `spdlog`, `nlohmann/json`, `simdjson`) across cold builds, warm rebuilds, and incremental touches:

```bash
# Benchmark fmt library across 3 iterations
python3 benchmarks/runner.py --library fmt --iterations 3

# Benchmark all supported libraries
python3 benchmarks/runner.py --library all
```

Results and comparison tables are saved to `benchmarks/results/latest_report.md`.

### 2. Lexer Throughput Microbenchmark
Measures raw FSM lexing throughput (MB/s) across synthetic source files up to 100,000 lines with high emoji densities:

```bash
cargo run --release --bin bench_lexer
```

### 3. Continuous Integration & Benchmarks (GitHub Actions)
A complete [`.github/workflows/ci.yml`](.github/workflows/ci.yml) workflow is configured:
- Automatically runs unit tests, acceptance tests, and LLDB breakpoint validation.
- Runs compiler benchmarks on pull requests and merges, rendering the results directly to the GitHub Actions Job Summary and publishing artifacts.
- For detailed local setup and adding new libraries, see [`benchmarks/README.md`](benchmarks/README.md).

### 3. Continuous Integration & Benchmarks (GitHub Actions)
A complete [`.github/workflows/ci.yml`](.github/workflows/ci.yml) workflow is configured:
- Automatically runs unit tests, acceptance tests, and LLDB breakpoint validation.
- Runs compiler benchmarks on pull requests and merges, rendering the results directly to the GitHub Actions Job Summary and publishing artifacts.
- For detailed local setup and adding new libraries, see [`benchmarks/README.md`](benchmarks/README.md).

---

## License

Dual-licensed under MIT or Apache-2.0.
