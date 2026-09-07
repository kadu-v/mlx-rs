extern crate cmake;

use cmake::Config;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use std::fs;

#[path = "../xtask/src/bindgen_config.rs"]
mod bindgen_config;

/// Find the clang runtime library path dynamically using xcrun
fn find_clang_rt_path() -> Option<String> {
    // Use xcrun to find the active toolchain path
    let output = Command::new("xcrun")
        .args(["--show-sdk-platform-path"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Get the developer directory which contains the toolchain
    let output = Command::new("xcode-select")
        .args(["--print-path"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let developer_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let toolchain_base = format!(
        "{}/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang",
        developer_dir
    );

    // Find the clang version directory (it varies by Xcode version)
    let clang_dir = std::fs::read_dir(&toolchain_base).ok()?;
    for entry in clang_dir.flatten() {
        let darwin_path = entry.path().join("lib/darwin");
        let clang_rt_lib = darwin_path.join("libclang_rt.osx.a");
        if clang_rt_lib.exists() {
            return Some(darwin_path.to_string_lossy().to_string());
        }
    }

    None
}

fn mlx_c_key(mlx_c_root: &Path) -> String {
    if mlx_c_root.join(".git").exists() {
        let output = Command::new("git")
            .arg("-C")
            .arg(mlx_c_root)
            .args(["rev-parse", "HEAD"])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let commit = String::from_utf8_lossy(&output.stdout);
                let commit = commit.trim();
                if commit.len() >= 12 && commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return commit[..12].to_owned();
                }
            }
        }
    }

    let mut files = bindgen_config::discover_headers(mlx_c_root)
        .expect("Unable to discover mlx-c headers for the metallib key");
    files.push(mlx_c_root.join("CMakeLists.txt"));
    files.sort();

    let mut hash = 0xcbf29ce484222325_u64;
    for file in files {
        let relative = file.strip_prefix(mlx_c_root).unwrap_or(&file);
        for byte in relative.to_string_lossy().bytes().chain([0]) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for byte in fs::read(&file).expect("Unable to hash mlx-c source for the metallib key") {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")[..12].to_owned()
}

fn metallib_dir(mlx_c_root: &Path, target: &str) -> PathBuf {
    if let Some(path) = env::var_os("MLX_RS_METAL_PATH") {
        return PathBuf::from(path);
    }

    let home =
        env::var_os("HOME").expect("HOME must be set when MLX_RS_METAL_PATH is not provided");
    // The key only identifies the mlx-c revision. Metal libraries are not
    // portable across platforms, so the target keeps macOS, iOS and simulator
    // builds of the same revision from overwriting each other.
    PathBuf::from(home)
        .join(".mlx")
        .join("lib")
        .join(mlx_c_key(mlx_c_root))
        .join(target)
}

/// Copy the AOT-compiled `mlx.metallib` to stable locations so packaging steps
/// (xcframework assembly, for instance) can pick it up. `OUT_DIR` is where a
/// downstream build looks when it cannot set an explicit path.
fn export_metallib(metallib: &Path) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy(metallib, out_dir.join("mlx.metallib"))
        .expect("failed to copy mlx.metallib to OUT_DIR");

    if let Ok(export_path) = env::var("MLX_METALLIB_EXPORT_PATH") {
        let export_path = PathBuf::from(export_path);
        if let Some(parent) = export_path.parent() {
            fs::create_dir_all(parent)
                .expect("failed to create the MLX_METALLIB_EXPORT_PATH directory");
        }
        fs::copy(metallib, &export_path)
            .expect("failed to copy mlx.metallib to MLX_METALLIB_EXPORT_PATH");
    }
}

fn build_and_link_mlx_c() {
    let mlx_c_root = Path::new("src/mlx-c");
    let target = env::var("TARGET").unwrap_or_default();
    let mut config = Config::new(mlx_c_root);
    config.very_verbose(true);
    config.define("CMAKE_INSTALL_PREFIX", ".");

    // Use Xcode's clang to ensure compatibility with the macOS SDK
    config.define("CMAKE_C_COMPILER", "/usr/bin/cc");
    config.define("CMAKE_CXX_COMPILER", "/usr/bin/c++");

    #[cfg(debug_assertions)]
    {
        config.define("CMAKE_BUILD_TYPE", "Debug");
    }

    #[cfg(not(debug_assertions))]
    {
        config.define("CMAKE_BUILD_TYPE", "Release");
    }

    // CMAKE_SYSTEM_NAME is "iOS" for both device and simulator, so mlx would
    // build the Metal backend against the device SDK on the simulator. The
    // simulator has no usable Metal backend here, so fall back to the CPU one.
    let is_ios = target.contains("apple-ios");
    let is_ios_sim = is_ios && (target.ends_with("-sim") || target.starts_with("x86_64"));
    let metal_enabled = cfg!(feature = "metal") && !is_ios_sim;

    config.define("MLX_BUILD_METAL", if metal_enabled { "ON" } else { "OFF" });
    config.define("MLX_BUILD_ACCELERATE", "OFF");

    if metal_enabled {
        let metallib_dir = metallib_dir(mlx_c_root, &target);
        fs::create_dir_all(&metallib_dir).expect("Unable to create the MLX metallib directory");
        config.define("MLX_METAL_PATH", &metallib_dir);

        // Same configuration as mlx-swift: JIT for most kernels plus a small
        // AOT mlx.metallib for the kernels that cannot be JIT-compiled.
        config.define("MLX_METAL_JIT", "ON");
    }

    if is_ios {
        // The cmake crate does not set CMAKE_OSX_DEPLOYMENT_TARGET for iOS
        // targets; without it the metallib min-OS floats up to the SDK version.
        if let Ok(deployment_target) = env::var("IPHONEOS_DEPLOYMENT_TARGET") {
            config.define("CMAKE_OSX_DEPLOYMENT_TARGET", &deployment_target);
        }
    }

    // SwiftPM bundle name to search for default.metallib at runtime, used when
    // mlx is linked into an app as a static library.
    if let Ok(bundle) = env::var("MLX_SWIFTPM_BUNDLE") {
        config.define("MLX_SWIFTPM_BUNDLE", &bundle);
    }

    // Point FetchContent at a local mlx checkout for development iteration.
    if let Ok(mlx_source_dir) = env::var("MLX_SOURCE_DIR") {
        config.define("FETCHCONTENT_SOURCE_DIR_MLX", &mlx_source_dir);
    }

    #[cfg(feature = "accelerate")]
    {
        config.define("MLX_BUILD_ACCELERATE", "ON");
    }

    // build the mlx-c project
    let dst = config.build();

    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    // mlx's GGUF io depends on the vendored gguflib archive, which cmake leaves
    // in the mlx build tree instead of installing next to libmlx.
    println!(
        "cargo:rustc-link-search=native={}/build/_deps/mlx-build/mlx/io",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=mlx");
    println!("cargo:rustc-link-lib=static=mlxc");
    println!("cargo:rustc-link-lib=static=gguflib");

    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=dylib=objc");
    println!("cargo:rustc-link-lib=framework=Foundation");

    if metal_enabled {
        println!("cargo:rustc-link-lib=framework=Metal");
        let metallib = metallib_dir(mlx_c_root, &target).join("mlx.metallib");
        if !metallib.exists() {
            println!(
                "cargo:warning=mlx.metallib was not created at {}; Metal operations may fail at runtime",
                metallib.display()
            );
        } else {
            export_metallib(&metallib);
        }
    }

    #[cfg(feature = "accelerate")]
    {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    // Link against Xcode's clang runtime for ___isPlatformVersionAtLeast symbol
    // This is needed on macOS 26+ where the bundled LLVM runtime may be outdated
    // See: https://github.com/conda-forge/llvmdev-feedstock/issues/244
    // libclang_rt.osx only ships macOS slices, so linking it into an iOS target
    // fails; iOS gets the symbol from its own SDK runtime instead.
    if !target.contains("apple-ios") {
        if let Some(clang_rt_path) = find_clang_rt_path() {
            println!("cargo:rustc-link-search={}", clang_rt_path);
            println!("cargo:rustc-link-lib=static=clang_rt.osx");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=MLX_RS_METAL_PATH");
    println!("cargo:rerun-if-env-changed=IPHONEOS_DEPLOYMENT_TARGET");
    println!("cargo:rerun-if-env-changed=MLX_SWIFTPM_BUNDLE");
    println!("cargo:rerun-if-env-changed=MLX_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=MLX_METALLIB_EXPORT_PATH");
    build_and_link_mlx_c();

    let mlx_c_root = PathBuf::from("src/mlx-c");
    let headers =
        bindgen_config::discover_headers(&mlx_c_root).expect("Unable to discover headers");
    for header in &headers {
        let relative =
            bindgen_config::relative_header(&mlx_c_root, header).expect("Unable to record header");
        println!(
            "cargo:rerun-if-changed={}",
            mlx_c_root.join(relative).display()
        );
    }
    let bindings = bindgen_config::builder(&mlx_c_root, &headers)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
