#!/usr/bin/env python3
"""
emocxx Benchmark Harness: Compares compilation performance across popular C++ libraries
with and without emocxx proxying.

Measures:
1. CMake configuration time
2. Clean build time (cold cache)
3. Zero-change incremental rebuild time (Ninja no-work-to-do check)
4. Incremental touch build time (single translation unit recompile)
"""

import argparse
import json
import math
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

LIBRARIES = {
    "fmt": {
        "url": "https://github.com/fmtlib/fmt.git",
        "tag": "10.2.1",
        "cmake_args": ["-DFMT_TEST=OFF", "-DFMT_DOC=OFF"],
        "touch_file": "src/format.cc",
    },
    "spdlog": {
        "url": "https://github.com/gabime/spdlog.git",
        "tag": "v1.13.0",
        "cmake_args": ["-DSPDLOG_BUILD_TESTS=OFF", "-DSPDLOG_BUILD_EXAMPLE=OFF"],
        "touch_file": "src/spdlog.cpp",
    },
    "json": {
        "url": "https://github.com/nlohmann/json.git",
        "tag": "v3.11.3",
        "cmake_args": ["-DJSON_BuildTests=OFF"],
        "touch_file": None,
    },
    "simdjson": {
        "url": "https://github.com/simdjson/simdjson.git",
        "tag": "v3.8.0",
        "cmake_args": ["-DSIMDJSON_DEVELOPER_MODE=OFF"],
        "touch_file": "src/simdjson.cpp",
    },
    "boost": {
        "local_path": "benchmarks/suites/boost_stress",
        "cmake_args": [],
        "touch_file": "src/mp11_metaprog.cpp",
    },
}

def run_cmd(cmd, cwd=None, env=None, capture=True):
    """Executes a command and returns (exit_code, wall_time_seconds, stdout, stderr)."""
    start = time.perf_counter()
    p = subprocess.run(
        cmd,
        cwd=cwd,
        env=env or os.environ.copy(),
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=True,
    )
    elapsed = time.perf_counter() - start
    return p.returncode, elapsed, p.stdout or "", p.stderr or ""

def stats(values):
    """Computes mean and stddev for a list of floats."""
    if not values:
        return 0.0, 0.0
    mean = sum(values) / len(values)
    variance = sum((x - mean) ** 2 for x in values) / len(values) if len(values) > 1 else 0.0
    return mean, math.sqrt(variance)

def ensure_cloned(lib_name, info, repos_dir):
    """Returns local path or shallow clones the library if not already present."""
    if "local_path" in info:
        root_dir = repos_dir.parent.parent
        return (root_dir / info["local_path"]).resolve()

    dest = repos_dir / lib_name
    if not dest.exists():
        print(f"[*] Cloning {lib_name} ({info['tag']})...")
        cmd = [
            "git", "clone", "--depth", "1",
            "--branch", info["tag"],
            info["url"], str(dest)
        ]
        ret, _, _, err = run_cmd(cmd)
        if ret != 0:
            raise RuntimeError(f"Failed to clone {lib_name}: {err}")
    return dest

def benchmark_scenario(lib_name, info, repo_dir, work_dir, mode, real_cxx, emocxx_bin, jobs, iterations):
    """Runs benchmark iterations for a specific library in baseline or emocxx mode."""
    build_dir = work_dir / f"build_{lib_name}_{mode}"

    cmake_times = []
    clean_build_times = []
    zero_change_times = []
    touch_times = []

    cmake_flags = info.get("cmake_args", [])
    jobs_flag = f"-j{jobs}"

    for it in range(1, iterations + 1):
        # 1. Clean build directory
        if build_dir.exists():
            shutil.rmtree(build_dir)
        build_dir.mkdir(parents=True, exist_ok=True)

        # Clear .emocxx_cache if emocxx mode
        cache_dir = repo_dir / ".emocxx_cache"
        if cache_dir.exists():
            shutil.rmtree(cache_dir)

        bench_env = os.environ.copy()
        bench_env["EMOCXX_REAL_CXX"] = real_cxx
        bench_env["REAL_CXX"] = real_cxx

        # Configure CMake flags based on mode
        cmd_cmake = [
            "cmake", "-S", str(repo_dir), "-B", str(build_dir), "-G", "Ninja"
        ]
        if mode == "baseline":
            cmd_cmake.extend([f"-DCMAKE_CXX_COMPILER={real_cxx}"])
        else:
            cmd_cmake.extend([
                f"-DCMAKE_CXX_COMPILER={emocxx_bin}",
                f"-DCMAKE_CXX_COMPILER_ARG1=--real-cxx={real_cxx}",
                f"-DREAL_CXX={real_cxx}",
            ])
        cmd_cmake.extend(cmake_flags)

        # Measure Scenario A: CMake Configure
        ret, t_cfg, out_cfg, err_cfg = run_cmd(cmd_cmake, env=bench_env)
        if ret != 0:
            details = err_cfg if err_cfg.strip() else out_cfg
            raise RuntimeError(f"CMake configure failed for {lib_name} ({mode}):\n{details}")
        cmake_times.append(t_cfg)

        # Measure Scenario B: Clean Build
        cmd_build = ["ninja", "-C", str(build_dir), jobs_flag]
        ret, t_clean, out_clean, err_clean = run_cmd(cmd_build, env=bench_env)
        if ret != 0:
            details = err_clean if err_clean.strip() else out_clean
            raise RuntimeError(f"Clean build failed for {lib_name} ({mode}):\n{details}")
        clean_build_times.append(t_clean)

        # Measure Scenario C: Zero-Change Rebuild
        ret, t_zero, out_zero, err_zero = run_cmd(["ninja", "-C", str(build_dir)], env=bench_env)
        if ret != 0:
            details = err_zero if err_zero.strip() else out_zero
            raise RuntimeError(f"Zero-change rebuild failed for {lib_name} ({mode}):\n{details}")
        zero_change_times.append(t_zero)

        # Measure Scenario D: Incremental 1-file Touch (if configured)
        touch_rel = info.get("touch_file")
        if touch_rel and (repo_dir / touch_rel).exists():
            target_file = repo_dir / touch_rel
            target_file.touch()
            ret, t_touch, out_touch, err_touch = run_cmd(cmd_build, env=bench_env)
            if ret != 0:
                details = err_touch if err_touch.strip() else out_touch
                raise RuntimeError(f"Touch rebuild failed for {lib_name} ({mode}):\n{details}")
            touch_times.append(t_touch)

    return {
        "cmake_cfg": stats(cmake_times),
        "clean_build": stats(clean_build_times),
        "zero_change": stats(zero_change_times),
        "touch_build": stats(touch_times) if touch_times else None,
    }

def main():
    parser = argparse.ArgumentParser(description="emocxx Compiler Overhead Benchmark")
    parser.add_argument("--library", choices=list(LIBRARIES.keys()) + ["all"], default="fmt",
                        help="Library to benchmark (default: fmt)")
    parser.add_argument("--iterations", type=int, default=3,
                        help="Number of iterations for statistical averaging (default: 3)")
    parser.add_argument("--real-cxx", default=os.getenv("REAL_CXX", "clang++"),
                        help="Path or name of real host compiler (default: clang++)")
    parser.add_argument("--emocxx-bin", default=None,
                        help="Path to compiled emocxx binary (default: target/release/emocxx)")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 4,
                        help="Parallel build jobs (default: logical CPU count)")
    args = parser.parse_args()

    root_dir = Path(__file__).resolve().parent.parent
    emocxx_bin = Path(args.emocxx_bin) if args.emocxx_bin else root_dir / "target" / "release" / "emocxx"
    if not emocxx_bin.exists():
        print(f"[!] Error: emocxx binary not found at {emocxx_bin}")
        print("    Please run `cargo build --release` first or supply --emocxx-bin.")
        sys.exit(1)

    real_cxx = shutil.which(args.real_cxx)
    if not real_cxx:
        real_cxx = args.real_cxx

    bench_work_dir = root_dir / ".benchmarks"
    repos_dir = bench_work_dir / "repos"
    results_dir = root_dir / "benchmarks" / "results"
    repos_dir.mkdir(parents=True, exist_ok=True)
    results_dir.mkdir(parents=True, exist_ok=True)

    targets = list(LIBRARIES.keys()) if args.library == "all" else [args.library]

    print("=================================================================")
    print("                    emocxx Benchmark Suite                       ")
    print(f" Host Compiler : {args.real_cxx}")
    print(f" emocxx Binary : {emocxx_bin}")
    print(f" Parallel Jobs : {args.jobs}")
    print(f" Iterations    : {args.iterations}")
    print("=================================================================\n")

    summary_rows = []

    for lib_name in targets:
        info = LIBRARIES[lib_name]
        print(f"\n--- Benchmarking: {lib_name} ---")
        repo_dir = ensure_cloned(lib_name, info, repos_dir)

        print(f"[*] Running Baseline ({real_cxx})...")
        base = benchmark_scenario(
            lib_name, info, repo_dir, bench_work_dir, "baseline",
            real_cxx, emocxx_bin, args.jobs, args.iterations
        )

        print("[*] Running emocxx Wrapper...")
        emo = benchmark_scenario(
            lib_name, info, repo_dir, bench_work_dir, "emocxx",
            real_cxx, emocxx_bin, args.jobs, args.iterations
        )

        def overhead(emo_val, base_val):
            if base_val <= 0:
                return 0.0
            return ((emo_val - base_val) / base_val) * 100.0

        clean_diff = emo["clean_build"][0] - base["clean_build"][0]
        clean_pct = overhead(emo["clean_build"][0], base["clean_build"][0])
        cfg_pct = overhead(emo["cmake_cfg"][0], base["cmake_cfg"][0])
        zero_diff = (emo["zero_change"][0] - base["zero_change"][0]) * 1000.0  # in ms

        summary_rows.append({
            "lib": lib_name,
            "base_clean": base["clean_build"][0],
            "emo_clean": emo["clean_build"][0],
            "clean_diff": clean_diff,
            "clean_pct": clean_pct,
            "base_cfg": base["cmake_cfg"][0],
            "emo_cfg": emo["cmake_cfg"][0],
            "cfg_pct": cfg_pct,
            "zero_diff_ms": zero_diff,
        })

    # Render Markdown Report
    print("\n\n========================= BENCHMARK RESULTS =========================")
    header_line = "| Library | Clean Baseline (s) | Clean emocxx (s) | Overhead (s) | Overhead (%) | Zero-Rebuild Delta (ms) |"
    sep_line = "| :--- | :--- | :--- | :--- | :--- | :--- |"
    print(header_line)
    print(sep_line)

    report_md = [
        "# emocxx Benchmark Report",
        f"**Host Compiler**: `{real_cxx}` | **Jobs**: `{args.jobs}` | **Iterations**: `{args.iterations}`\n",
        header_line,
        sep_line,
    ]

    for row in summary_rows:
        line = f"| **{row['lib']}** | {row['base_clean']:.2f}s | {row['emo_clean']:.2f}s | +{row['clean_diff']:.2f}s | **{row['clean_pct']:+.1f}%** | {row['zero_diff_ms']:+.1f}ms |"
        report_md.append(line)
        print(line)

    report_path = results_dir / "latest_report.md"
    report_path.write_text("\n".join(report_md) + "\n")
    print(f"\nSaved report to {report_path}")

if __name__ == "__main__":
    main()

