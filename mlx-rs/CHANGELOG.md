# Changelog

## 0.26.0-ios.1

`oxiglade/mlx-rs` の未リリース版 (`d1440761`) をベースにしたフォークのリリース。
`v0.25.3-ios.1` から取り込んだ upstream の主な変更:

- metallib distribution / rope shape / padded-buffer guards (#365)
- memory controls, contiguous, optional rms_norm weight (#367)
- gguf / math / index-update / verification utils の各 cohort (#362, #363, #364)
- mlx-c を `c74db530` に更新（MLX v0.30.6 → v0.32.2）、MSRV を 1.88.0 に。
  MLX 側の差分として、NAX カーネルが v0.32.2 のゲート（Metal >= 400 かつ macOS SDK >= 26.2）により
  iOS ビルドから除外され、AOT `mlx.metallib` は 2.7MB → 1.4MB になった（AOT/JIT の振り分け変更）。
  iOS ビルドの Metal 有効化・SDK・deployment target・backend object は v0.25.3-ios.1 と同一
- org のリネームに追従 (`oxideai` → `oxiglade`)

フォーク独自の変更:

- iOS ターゲットで `clang_rt.osx` をリンクしない
- iOS / シミュレータ向けの cmake 設定 (`MLX_METAL_JIT`, `CMAKE_OSX_DEPLOYMENT_TARGET`,
  `MLX_SWIFTPM_BUNDLE`, `MLX_SOURCE_DIR`)。シミュレータでは Metal を強制 OFF
- `mlx.metallib` の出力先にターゲットトリプルを追加。macOS と iOS が同じパスを
  奪い合うのを防ぐ
- `mlx.metallib` を `OUT_DIR` と `MLX_METALLIB_EXPORT_PATH` にエクスポート
- submodule を `kadu-v/mlx-c` (`v0.6.0-ios.1`) に向ける。その先は
  `kadu-v/mlx` (`v0.32.2-ios.1`) = upstream v0.32.2 + iOS Metal cross-compilation
  ([ml-explore/mlx#3915](https://github.com/ml-explore/mlx/issues/3915))。
  v0.25.3-ios.1 で cherry-pick していた #3617 は v0.32.0 以降 upstream に含まれるため不要になった

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
