# Portability and source builds

The project targets native builds for macOS Apple Silicon, macOS Intel,
Windows x64, and Linux x64. Semantic search always has a CPU path. Apple
Silicon may use Metal when `--device auto` succeeds; `--device cpu` provides a
portable fallback.

The published Linux x64 archive is built natively on Ubuntu 22.04 and targets
glibc 2.35 or later. Build from source on older Linux systems or environments
with a different C/C++ runtime.

Source builds require the pinned Rust toolchain in `rust-toolchain.toml`, CMake,
and a supported C++ compiler for the embedded llama.cpp backend. The project
locks Rust dependencies in `Cargo.lock`; release builds must not use `target-cpu=native`,
so an archive is not accidentally limited to its build machine.

Use the same local validation command on every platform:

```sh
cargo xtask ci
```

Windows users can run the executable as `jgrep.exe`; PowerShell pipelines work
with the same stdin behavior as other shells. For path filtering, prefer
`--include '*.ext'` with `-r` rather than relying on shell wildcard expansion,
which differs across shells.
