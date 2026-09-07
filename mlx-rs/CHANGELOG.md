# Changelog

## 0.26.0-ios.1

`oxiglade/mlx-rs` の未リリース版 (`d1440761`) をベースにしたフォークのリリース。
`v0.25.3-ios.1` から取り込んだ upstream の主な変更:

- metallib distribution / rope shape / padded-buffer guards (#365)
- memory controls, contiguous, optional rms_norm weight (#367)
- gguf / math / index-update / verification utils の各 cohort (#362, #363, #364)
- mlx-c を `c74db530` (MLX v0.32.2) に更新、MSRV を 1.88.0 に
- org のリネームに追従 (`oxideai` → `oxiglade`)

フォーク独自の変更:

- iOS ターゲットで `clang_rt.osx` をリンクしない
- iOS / シミュレータ向けの cmake 設定 (`MLX_METAL_JIT`, `CMAKE_OSX_DEPLOYMENT_TARGET`,
  `MLX_SWIFTPM_BUNDLE`, `MLX_SOURCE_DIR`)。シミュレータでは Metal を強制 OFF
- `mlx.metallib` の出力先にターゲットトリプルを追加。macOS と iOS が同じパスを
  奪い合うのを防ぐ
- `mlx.metallib` を `OUT_DIR` と `MLX_METALLIB_EXPORT_PATH` にエクスポート
- submodule を `kadu-v/mlx-c` (`v0.6.0-ios.1`) に向ける。その先は
  `kadu-v/mlx` (`v0.32.2-ios.1`)

## 0.25.3

- @dshan4585 Prevent premature destructuring of closures & Add atan2 (#286)
- @Vlad-Shcherbina Fix not one but two leaks related to gradients (#296)
- @scttfrdmn Fix: Add missing Float64 pattern in safetensors conversion (#295)
- @Vlad-Shcherbina Add missing #[param] attributes to InstanceNorm (#300)

## 0.25.2

- Introduce initial support for mlx-lm
  - impl `Parameter` trait for `Option<T>` where `T: ModuleParameters`
  - Add `finfo_max` and `finfo_min`
  - impl `Quantizable` for `Option<T>` where `T: Quantizable`

## 0.25.1

- Fix bug with `index_mut`

## 0.25.0

- Update `mlx-c` to version "0.2.0" and changes function signatures to
  match the new API
- Update `thiserror` to version "2"
- Fix wrong states number in `compile_with_state`
- Remove unnecessary evaluation in fft ops

## 0.23.0

- Update `mlx-c` to "0.1.2"
- Added `dilation` and `groups` parameters to the convolution layer

## 0.21.1

- Fix `mlx-sys` dependency to patch version in workspace

## 0.21.0

- Initial feature-complete release
