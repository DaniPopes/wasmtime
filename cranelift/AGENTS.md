# Cranelift Agent Guide

## Build & Test Commands
- Build: `cargo build -p cranelift-*`
- Test: `cargo nextest -p cranelift-*`
- Single test: `cargo nextest -p cranelift-* -- test_name`
- Run filetest: `cargo run -- test filetests/filetests/runtests/foo.clif`
- Format: `cargo +nightly fmt --all`
- Lint: `cargo clippy -p cranelift-* --all-features --all-targets`

## Architecture
- **codegen/**: Core compiler - IR, instruction selection (ISLE), register allocation, emission
- **isle/**: DSL compiler for instruction selection rules (`.isle` → Rust at build time)
- **frontend/**: IR builder API for constructing Cranelift IR
- **entity/**: Type-safe entity maps (`PrimaryMap`, `SecondaryMap`, `EntityRef` trait)
- **machinst/**: Machine instruction backend framework (VCode, lowering, ABI, buffer)
- **filetests/**: End-to-end tests using `.clif` IR files with filecheck directives
- **jit/**, **module/**, **object/**: Higher-level compilation APIs
- Backends in `codegen/src/isa/`: x64, aarch64, riscv64, s390x, pulley

## Code Style
- `#![no_std]}` in core crates; use `alloc::` (Vec, String, Box) and `core::` (fmt, ops)
- Imports: `crate::` first, then `alloc`/`core`, then external crates
- Entity IDs: `struct MyEntity(u32); entity_impl!(MyEntity, "ME");` with `PrimaryMap`/`SecondaryMap`
- Error handling: `CodegenResult<T>` for fallible ops; no panics in production paths
- ISLE for lowering rules (`lower.isle`); Rust helpers for side effects only
- Tests: unit tests (`#[cfg(test)]`) for logic; filetests (`.clif`) for codegen behavior
- No `std::` without `#[cfg(feature = "std")]`; keep core crates `no_std`-compatible
