# フォーク運用ガイド

`kadu-v/mlx-rs` は [`oxiglade/mlx-rs`](https://github.com/oxiglade/mlx-rs)（旧 `oxideai/mlx-rs`）の
フォークで、iOS 向け Metal ビルドを通すためのパッチを載せている。

このドキュメントはブランチの役割・バージョン規則・upstream 追従の手順をまとめたもの。
upstream には無いファイルなので、rebase で衝突しない。

## フォーク連鎖

iOS 対応は3つのリポジトリにまたがる。下流から順に:

| リポジトリ | upstream | フォークが持つもの |
|---|---|---|
| `kadu-v/mlx-rs` | `oxiglade/mlx-rs` | `mlx-sys/build.rs` の iOS 対応、submodule の向き先 |
| `kadu-v/mlx-c` | `ml-explore/mlx-c` | `CMakeLists.txt` の FetchContent を `kadu-v/mlx` に向けるだけ（機能変更なし） |
| `kadu-v/mlx` | `ml-explore/mlx` | Metal ツールチェーンの SDK 選択（`xcrun -sdk iphoneos` 等）と `MLX_SWIFTPM_BUNDLE` |

更新は必ず **`mlx` → `mlx-c` → `mlx-rs`** の順に行い、下流はタグ参照で上流を pin する。

## ブランチ

3リポジトリとも同じ構造にしてある。

```
main                    upstream の完全ミラー。fast-forward のみ。独自コミットは絶対に置かない
  └ ios/<base>          upstream の上に自前パッチを載せた統合ブランチ。追従時に rebase + force-push
release/<line>-ios      出荷ライン。upstream は取り込まず hotfix のみ。rebase 禁止・force-push 禁止
```

`mlx-rs` の現行ブランチ:

- `main` — `upstream/main` のミラー
- `ios/main` — 自前パッチ 4 コミット（デフォルトブランチ）
- `release/0.25.x-ios` — `v0.25.3-ios.1` から切った出荷ライン
- `backup/pre-restructure-main` — 再編前の `main` のバックアップ

`mlx` は `ios/v0.32.2`、`mlx-c` は `ios/c74db530` が現行の統合ブランチ。

### remote

```
origin    git@github.com:kadu-v/<repo>.git
upstream  https://github.com/<upstream-org>/<repo>.git   (push は無効化してある)
```

## バージョン / タグ規則

### mlx-rs

semver 解決を受けるのは `mlx-rs` だけなので、順序が単調に増えることを最優先する。
**「まだ出ていない次の upstream 版のプレリリース」** として名前を付ける。

| ライン | base の決め方 | タグ |
|---|---|---|
| 統合 `ios/main` | upstream 最新**リリース**版の **minor + 1** | `v0.26.0-ios.1`, `.2`, … |
| メンテ `release/0.25.x-ios` | 取り込み済み upstream リリース版の **patch + 1** | `v0.25.4-ios.1`, `.2`, … |

```
0.25.3 < 0.25.3-ios.1 (旧) < 0.25.4-ios.1 < 0.25.4 < 0.26.0-ios.1 < 0.26.0
```

- 2 ラインの base は構造的に衝突しない（メンテは patch+1、統合は minor+1）。
- upstream がリリースしたら、次の統合リリースから base を「新リリース版の minor+1」に更新する。
- **base は決して下げない。** upstream が予測と違う版を出しても、既発行タグより小さい base は使わない。
- プレリリース識別子が付くので、upstream の実リリースとタグ名が衝突することはない。

### mlx / mlx-c

CMake が SHA / タグで直接引くだけで semver 解決を受けないので、可読性を優先して
**「載せた upstream タグ + `-ios.N`」** とする（`v0.32.2-ios.1`, `v0.6.0-ios.1`）。
`mlx-rs` と規則が違うのは意図的。

## upstream 追従の手順（mlx-rs）

```bash
git fetch upstream --tags
git checkout main && git merge --ff-only upstream/main && git push origin main

git checkout ios/main
git rebase main
```

`mlx-sys/build.rs` で衝突したら次の方針で解決する。

- upstream の `mlx_c_key()` / `metallib_dir()` / `MLX_RS_METAL_PATH` の仕組みは土台として採用する。
- `metallib_dir()` の既定パスに付けたターゲットトリプルは残す。これが無いと macOS と iOS の
  `mlx.metallib` が同じ場所を奪い合う。
- upstream が `#[cfg(feature = "metal")]` に戻していたら、`metal_enabled` による実行時判定に置き換え直す。
  iOS シミュレータで Metal を強制 OFF にするために必要。

続いて検証（下記）を通してから:

```bash
# バージョンを上げる。4 箇所を同時に直すこと（後述）
$EDITOR Cargo.toml
$EDITOR mlx-rs/CHANGELOG.md
git commit -am "chore: release v0.26.0-ios.2"
git tag v0.26.0-ios.2
git push --force-with-lease origin ios/main
git push origin v0.26.0-ios.2
```

### バージョンを上げるときに直す 4 箇所

プレリリース識別子は caret 要求（`"0.25"`）に一致しないため、ワークスペース内依存は
`=` 固定にする。1 箇所でも忘れると `cargo` がパス依存のバージョン不一致でエラーになる。

1. `Cargo.toml` の `[workspace.package] version`
2. `[workspace.dependencies] mlx-rs`
3. `[workspace.dependencies] mlx-macros`
4. `[workspace.dependencies] mlx-internal-macros`

`mlx-sys` は upstream の方針（crate のバージョンは pin した mlx-c / MLX とは独立）に従って
据え置く。識別は git タグで足りる。

## hotfix の手順

出荷済みのタグに当てる修正は `release/*` に直接コミットする。**upstream は取り込まない。**

```bash
git checkout release/0.25.x-ios
# 修正してコミット
git tag v0.25.4-ios.2
git push origin release/0.25.x-ios v0.25.4-ios.2
```

このブランチは rebase も force-push もしない。消費側が pin しているタグの再現性を壊すため。

## フォーク連鎖を更新する手順

```bash
# 1. mlx: 新しい upstream タグの上にパッチを載せ替える
git -C ../mlx fetch upstream --tags
git -C ../mlx checkout -b ios/vX.Y.Z vX.Y.Z
git -C ../mlx cherry-pick <iOS パッチ>
git -C ../mlx tag vX.Y.Z-ios.1 && git -C ../mlx push origin ios/vX.Y.Z vX.Y.Z-ios.1

# 2. mlx-c: upstream mlx-rs が pin しているコミットの上に、FetchContent の
#    向き先を 1 で打ったタグに変える commit を載せる
#    → タグ vA.B.C-ios.N を打って push

# 3. mlx-rs: submodule ポインタを 2 のタグへ更新
```

**上流を pin するときは必ずタグを使い、ブランチ先端の SHA を直接指さない。**
ブランチを rebase すると SHA が到達不能になり、過去のタグから `git submodule update` が
できなくなる。

## 壊してはいけない契約

`say2-sdk` の xtask が以下の環境変数でこの crate をドライブしている。名前と意味を変えないこと。

| 環境変数 | 意味 |
|---|---|
| `MLX_SWIFTPM_BUNDLE` | 実行時に `default.metallib` を探す SwiftPM リソースバンドル名 |
| `MLX_METALLIB_EXPORT_PATH` | AOT コンパイル済み `mlx.metallib` のコピー先 |
| `IPHONEOS_DEPLOYMENT_TARGET` | metallib の min-OS（未設定だと SDK 版に浮く） |

`mlx.metallib` は `OUT_DIR` にも必ずコピーする。`say2-sdk` は build script が
fingerprint キャッシュでスキップされた場合に `build/mlx-sys-*/out/mlx.metallib` を探す。

## 検証

```bash
cargo build -p mlx-sys --target aarch64-apple-darwin
cargo build -p mlx-sys --target aarch64-apple-ios
cargo build -p mlx-sys --target aarch64-apple-ios-sim   # Metal は OFF になること
cargo test --all -- --test-threads=1

MLX_SWIFTPM_BUNDLE=Say2SDK_Say2SDK \
MLX_METALLIB_EXPORT_PATH=/tmp/verify/mlx.metallib \
IPHONEOS_DEPLOYMENT_TARGET=17.0 \
  cargo build -p mlx-sys --target aarch64-apple-ios
test -f /tmp/verify/mlx.metallib
```

macOS と iOS を続けてビルドし、`~/.mlx/lib/<key>/` 配下がターゲット別に分かれていること、
互いに上書きしていないことも確認する。
