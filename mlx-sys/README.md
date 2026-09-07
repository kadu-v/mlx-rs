# mlx-sys

Rust bindings to the mlx-c API. Generated using bindgen.

The crate version is independent of its native source tuple. This revision targets the
`kadu-v/mlx-c` fork at `v0.6.0-ios.1` (upstream mlx-c
`c74db5307cc8ce122f48d97ef951b30578674e7f` plus a redirect), whose CMake configuration pins the
`kadu-v/mlx` fork at `v0.32.2-ios.1` (upstream MLX `v0.32.2` plus iOS Metal cross-compilation
support). See `FORK.md` for why these forks exist.

## Metal library location

Metal builds place `mlx.metallib` in `~/.mlx/lib/<mlx-c-key>/<target>/`, where `<mlx-c-key>` is
the first 12 characters of the pinned mlx-c commit and `<target>` is the Rust target triple.
Metal libraries are not portable across platforms, so the target keeps macOS, iOS and simulator
builds of the same revision from overwriting each other. Packaged source without Git metadata uses a
deterministic hash of the mlx-c headers and CMake configuration instead. The stable location
allows binaries produced by `cargo install` to keep loading the library after Cargo removes its
temporary build directory.

Set `MLX_RS_METAL_PATH` to use a different directory verbatim as CMake's `MLX_METAL_PATH`.
When this override is set, the build does not read or write `HOME`, which supports sandboxed and
Nix builds.
