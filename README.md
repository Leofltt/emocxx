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
- **Emoji Filename & Include Support**: Source files (`🚀.cpp`) and headers (`📦.hpp`) with emojis in their names are fully supported. Includes like `#include "📦.hpp"` are sanitized to ASCII aliases for host compiler compatibility while depfiles and `#line` directives preserve original emoji filenames for build tools and debuggers.
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

### Full C++ Language Element Support

Because `emocxx` operates at the lexical token boundary, emojis can be used anywhere an identifier is valid in C++:

```cpp
// 1. Classes with Constructors, Destructors, and Methods
class 🤖 {
public:
    🤖() : 🔋(100) {}
    ~🤖() {}
    void ⚡() { 🔋 += 10; }
private:
    int 🔋;
};

// 2. Namespaces & Scoped Calls
namespace 🌌 {
    void 🚀() {}
}
// Call: 🌌::🚀();

// 3. Enum Classes
enum class 🚦 { 🔴, 🟡, 🟢 };
auto signal = 🚦::🟢;

// 4. Templates & Concepts (C++20)
template<typename 📦>
concept 📏 = requires(📦 item) { item.📐(); };

template<📏 📦>
class 🏭 { 📦 product; };

// 5. User-Defined Literals (UDL)
constexpr long double operator""_🚀(long double km) { return km * 1000.0; }
auto distance = 5.0_🚀;
```

---

## Architectural Limits & Boundaries

Because `emocxx` operates as a transparent compiler proxy and source transpiler rather than a patched compiler fork, it has specific structural boundaries:

| Language Feature / Use Case | Status | Explanation & Behavior |
| :--- | :---: | :--- |
| **Variables, Structs, Classes** | ✅ **Fully Supported** | Mangles deterministically to valid C++ identifiers (`_EMO_<hex>_`). |
| **Functions, Methods, Constructors** | ✅ **Fully Supported** | Compiles to standard Itanium/MSVC mangled symbols. |
| **Namespaces & Enum Classes** | ✅ **Fully Supported** | Scoped access (e.g. `🌌::🚀()`, `🚦::🟢`) works natively. |
| **Templates & Concepts (C++20)** | ✅ **Fully Supported** | Valid across template parameter and concept constraint positions. |
| **User-Defined Literals (`42_🚀`)** | ✅ **Fully Supported** | Supported via `operator""__EMO_..._` syntax. |
| **Filenames & `#include "📦.hpp"`** | ✅ **Fully Supported** | Mirrored into shadow cache with ASCII aliases; `#line 1` preserves file names. |
| **Debugger Stepping (GDB/LLDB)** | ✅ **Fully Supported** | Breakpoints set by source file and line (`b 🚀.cpp:9`) resolve 1:1. |
| **Debugger Variable Print** | ⚠️ **Mangled Name** | LLDB/GDB prints `_EMO_1f680_ = 42` instead of `🚀 = 42` without a demangler script. |
| **Macro Stringification (`#x`)** | ⚠️ **Mangled Name** | `#define STR(x) #x` evaluates `STR(🚀)` to `"_EMO_1f680_"`, not `"🚀"`. |
| **Compiler Introspection (`__func__`)**| ⚠️ **Mangled Name** | `__func__` and `std::source_location` report the mangled ASCII symbol name. |
| **Custom Emoji Operators (`a ➕ b`)** | ❌ **Unsupported** | C++ operator grammar is a fixed keyword set; emojis cannot create new operators. |
| **Public API Headers (Unwrapped)** | ❌ **Unsupported** | External consumers compiling without `emocxx` will hit compiler lexical errors. |

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
cmake -B build \
  -DCMAKE_TOOLCHAIN_FILE=$PWD/cmake/emocxx-toolchain.cmake \
  -DREAL_CXX=clang++

cmake --build build
```

### Method 2: Direct Command Line Invocation
```bash
cmake -B build \
  -DCMAKE_CXX_COMPILER=$PWD/target/release/emocxx \
  -DREAL_CXX=clang++

cmake --build build
```

---

## Running Acceptance Tests & Debugger Validation

1. Run unit tests:
```bash
cargo test
```

2. Build release binary:
```bash
cargo build --release
```

3. Configure and build the acceptance test:
```bash
cmake -B tests/acceptance/build -S tests/acceptance \
  -DCMAKE_TOOLCHAIN_FILE=$PWD/cmake/emocxx-toolchain.cmake \
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

Creative Commons 0 1.0 Universal license free for all to use and have fun with
