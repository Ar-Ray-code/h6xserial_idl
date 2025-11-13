# Rust Coding Guidelines for h6xserial_idl

本プロジェクトのRustコーディング規約を定義します。

## 目次

1. [命名規則](#命名規則)
2. [コード構造](#コード構造)
3. [エラーハンドリング](#エラーハンドリング)
4. [ドキュメント](#ドキュメント)
5. [テスト](#テスト)
6. [パフォーマンス](#パフォーマンス)
7. [依存関係](#依存関係)

## 命名規則

### 関数名
- **公開API**: 明確で説明的な名前を使用
  ```rust
  pub fn parse_messages(...) -> Result<...>
  pub fn generate(...) -> Result<String>
  ```

- **内部関数**: 目的を明確に示す
  ```rust
  fn parse_message_definition(...) -> Result<MessageDefinition>
  fn get_optional_endian(...) -> Result<Option<Endian>>
  ```

- **変換関数**: `to_*` プレフィックスを使用
  ```rust
  pub(crate) fn to_snake_case(name: &str) -> String
  pub(crate) fn to_macro_ident(name: &str) -> String
  ```

### 型名
- **構造体**: PascalCase、明確な名詞
  ```rust
  pub struct MessageDefinition { ... }
  pub struct ScalarSpec { ... }
  ```

- **列挙型**: PascalCase、状態や種類を表す
  ```rust
  pub enum MessageBody { ... }
  pub enum PrimitiveType { ... }
  ```

### 定数
- **SCREAMING_SNAKE_CASE**: グローバル定数
  ```rust
  const TEMPLATE_FILES: &[&str] = &[...];
  ```

## コード構造

### モジュール構成
```
src/
├── lib.rs          # 公開API、型定義、パーサー
├── main.rs         # CLIエントリーポイント
├── emit_c.rs       # C言語コード生成
└── msg_template/   # 言語別テンプレート
    └── c/          # C言語ヘルパー
```

### 可視性
- **公開型**: テスト可能性のため、必要な型は`pub`に
  ```rust
  pub struct Metadata { ... }
  pub enum PrimitiveType { ... }
  ```

- **内部実装**: `pub(crate)`または`private`
  ```rust
  pub(crate) fn to_snake_case(...) -> String
  fn parse_message_definition(...) -> Result<...>
  ```

### 関数の長さ
- 1関数は50行以内を目安に
- 複雑な処理は小さな関数に分割
- 単一責任の原則を守る

## エラーハンドリング

### anyhowの使用
```rust
use anyhow::{Context, Result, bail};

fn parse_foo(...) -> Result<Foo> {
    let value = map.get("key")
        .with_context(|| format!("missing required field 'key'"))?;

    if !is_valid(value) {
        bail!("invalid value for 'key': {}", value);
    }

    Ok(Foo { ... })
}
```

### エラーメッセージ
- ユーザーフレンドリーなメッセージ
- コンテキスト情報を含める
- 解決方法のヒントを提供

```rust
// Good
.with_context(|| format!("message '{}' requires 'max_length' field", name))?

// Bad
.context("missing field")?
```

## ドキュメント

### パブリックAPI
```rust
/// Parses JSON message definitions into internal structures.
///
/// # Arguments
/// * `map` - JSON object containing message definitions
///
/// # Returns
/// * `Ok((metadata, messages))` - Parsed metadata and message list
/// * `Err(...)` - Parse error with context
///
/// # Example
/// ```
/// let json = serde_json::from_str(json_str)?;
/// let obj = json.as_object().unwrap();
/// let (metadata, messages) = parse_messages(obj)?;
/// ```
pub fn parse_messages(map: &Map<String, Value>) -> Result<(Metadata, Vec<MessageDefinition>)>
```

### 内部実装
```rust
// Converts a name to snake_case format.
// Non-alphanumeric characters are replaced with underscores.
fn to_snake_case(name: &str) -> String
```

## テスト

### ユニットテスト
- 各関数に対して網羅的なテスト
- エッジケースを含める
- 失敗ケースもテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("HelloWorld"), "helloworld");
        assert_eq!(to_snake_case(""), "msg");  // Edge case
    }

    #[test]
    fn test_array_without_max_length_fails() {
        let result = parse_messages(invalid_json);
        assert!(result.is_err());  // Failure case
    }
}
```

### 統合テスト
- `tests/` ディレクトリに配置
- エンドツーエンドのワークフローをテスト
- 一時ファイルは`tempfile`クレートを使用

```rust
#[test]
fn test_generate_c_header_from_example_json() {
    let temp_dir = TempDir::new().unwrap();
    // ... テストロジック
}
```

## パフォーマンス

### メモリ効率
- 不要なクローンを避ける
- 参照を活用
- ムーブセマンティクスを理解

```rust
// Good: 参照を使用
fn process_message(msg: &MessageDefinition) -> String

// Bad: 不要なクローン
fn process_message(msg: MessageDefinition) -> String
```

### 文字列操作
- `String::with_capacity`で事前割り当て
- `format!`より`write!`マクロを検討
- 小さな文字列は`&str`を使用

```rust
let mut result = String::with_capacity(estimated_size);
write!(&mut result, "value: {}", x).unwrap();
```

## 依存関係

### 最小限の依存
現在の依存関係：
```toml
[dependencies]
anyhow = "1.0"           # エラーハンドリング
serde = { version = "1.0", features = ["derive"] }  # シリアライゼーション
serde_json = "1.0"       # JSON解析

[dev-dependencies]
tempfile = "3.8"         # テスト用一時ファイル
```

### 新規依存の追加基準
- 必須の機能である
- よくメンテナンスされている
- 軽量である
- 代替手段がない

## コード品質

### Clippy
- すべての警告に対応
- `#[allow(...)]`の使用は最小限に
- 使用する場合はコメントで理由を説明

```rust
// This function is reserved for future language generators (C++, Python, etc.)
#[allow(dead_code)]
pub(crate) fn to_pascal_case(name: &str) -> String
```

### Rustfmt
- すべてのコードをフォーマット
- デフォルト設定を使用
- コミット前に`cargo fmt`を実行

### コミット前チェックリスト
```bash
cargo fmt --check        # フォーマット確認
cargo clippy -- -D warnings  # 警告なし
cargo test              # すべてのテスト通過
cargo build --release   # リリースビルド成功
```

## 具体的なパターン

### Result型の伝播
```rust
// Good: ?演算子を使用
fn parse_foo() -> Result<Foo> {
    let value = get_value()?;
    let processed = process(value)?;
    Ok(Foo { processed })
}

// Bad: unwrap()の使用
fn parse_foo() -> Foo {
    let value = get_value().unwrap();
    Foo { value }
}
```

### パターンマッチング
```rust
// Good: 網羅的なマッチ
match message_body {
    MessageBody::Scalar(spec) => generate_scalar(spec),
    MessageBody::Array(spec) => generate_array(spec),
    MessageBody::Struct(spec) => generate_struct(spec),
}

// Bad: デフォルトケースの乱用
match message_body {
    MessageBody::Scalar(spec) => generate_scalar(spec),
    _ => panic!("unexpected type"),
}
```

### イテレータの活用
```rust
// Good: イテレータチェーン
messages.iter()
    .filter(|m| m.packet_id > 10)
    .map(|m| m.name.clone())
    .collect()

// Bad: 手動ループ
let mut names = Vec::new();
for msg in &messages {
    if msg.packet_id > 10 {
        names.push(msg.name.clone());
    }
}
```

## セキュリティ

### 入力検証
- すべての外部入力を検証
- 範囲チェックを実施
- サニタイズが必要な場合は実施

```rust
if max_length == 0 || max_length > MAX_ARRAY_SIZE {
    bail!("max_length must be between 1 and {}", MAX_ARRAY_SIZE);
}
```

### パスの安全性
- パストラバーサル攻撃を防ぐ
- 絶対パスと相対パスを適切に処理

```rust
let output_path = PathBuf::from(user_input);
if !output_path.starts_with(&safe_base_dir) {
    bail!("invalid output path");
}
```

## 将来の拡張性

### 言語追加への対応
```rust
pub enum TargetLanguage {
    C,
    // Future: Cpp, Python, Rust
}

impl TargetLanguage {
    fn template_subdir(self) -> &'static str {
        match self {
            TargetLanguage::C => "c",
            // TargetLanguage::Cpp => "cpp",
        }
    }
}
```

### プラグインアーキテクチャ
- コード生成器を分離（`emit_*.rs`）
- 共通インターフェースを定義
- 各言語の実装を独立させる

## まとめ

このガイドラインは：
- コードの一貫性を保つ
- メンテナンス性を向上させる
- バグを減らす
- 新しい開発者のオンボーディングを容易にする

ことを目的としています。

**すべてのコントリビューターはこのガイドラインに従ってください。**
