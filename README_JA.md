# h6xserial_idl

[![Testing](https://github.com/Ar-Ray-code/h6xserial_idl/actions/workflows/test.yml/badge.svg)](https://github.com/Ar-Ray-code/h6xserial_idl/actions/workflows/test.yml)

Rust 製のコードジェネレーターです。`msgs/intermediate_msg.json` のような JSON 中間表現を読み込み、C99 向けヘッダーのシリアライザ／デシリアライザを生成します。

対応言語
- C99
- C++ (TODO)
- Python (TODO)
- Rust (TODO)

## 仕組み

1. JSON を読み込み、メタデータ・メッセージ定義を内部構造体にパースします。
2. 指定された言語に応じてテンプレートとコード生成器を切替えます。
3. C 向けはヘッダガード付きの C99 ヘッダーを生成し、エンコード/デコード関数と補助関数を出力します。
4. 出力先が存在しない場合はディレクトリを作成し、ファイルを書き出します。

テンプレートは `src/msg_template/<lang>/` に配置されており、言語ごとの補助関数や共通コードはこのディレクトリから読み込まれます。

## 使いかた

### 実行方法

```bash
# C99 ヘッダーを生成（デフォルト）
cargo run

# ドキュメントを生成
cargo run -- --export_docs

# 入力 / 出力パスを指定
cargo run -- [--export_docs] [入力JSON] [出力パス]
```

- 言語を省略すると `c` がデフォルトです。
- 入力パスを省略すると `msgs/intermediate_msg.json`（無い場合は `../msgs/intermediate_msg.json`）を探します。
- 出力パスを省略すると言語ごとの既定パスに書き込みます（C は `generated_c/seridl_generated_messages.h`、ドキュメントは `docs/COMMANDS.md`）。

### ドキュメント生成

`--export_docs` フラグを使用すると、コマンド定義のドキュメントを Markdown 形式で自動生成できます。

```bash
# デフォルト位置にドキュメントを生成（docs/COMMANDS.md）
cargo run -- --export_docs

# カスタムの入力・出力パスを指定
cargo run -- --export_docs msgs/intermediate_msg.json docs/MY_COMMANDS.md
```

生成されるドキュメントには以下が含まれます：
- packet ID でソートされたコマンド定義テーブル
- Base Commands (0~19) と Custom Commands (20+) のセクション
- コマンド名、値、説明が読みやすい形式で記載されます

出力例：

```markdown
## Base Commands (0~19)

| Command | Value | Description |
|---------|-------|-------------|
| `CMD_PING` | 0 | Ping/keep-alive command |
| `CMD_INTERNAL_LED_ON_OFF` | 1 | Toggle internal LED |
| `CMD_REBOOT_DEVICE` | 2 | Reboot target device |
...
```

### `/usr/local/bin` へのインストール

`h6xserial_idl` を常に `cargo run` 経由で実行しなくても済むように、次の手順でバイナリを `/usr/local/bin` に配置できます。

```bash
cd h6xserial_idl
cargo build --release
sudo install -m 0755 target/release/h6xserial_idl /usr/local/bin/h6xserial_idl
```

`install` コマンドが存在しない環境では、代わりに `sudo cp` と `sudo chmod 755` を使ってください。インストール後は、どこからでも `h6xserial_idl` を直接実行できます。

### 中間表現について

`msgs/intermediate_msg.json` のような JSON でメッセージを定義します。例:

```json
{
  "version": "0.0.1",
  "max_address": 255,
  "get_temperatures": {
    "packet_id": 20,
    "msg_type": "float32",
    "array": true,
    "endianess": "big",
    "max_length": 4,
    "msg_desc": "Get temperature readings"
  }
}
```

- `msg_type` が `struct` の場合は `fields` オブジェクトにフィールドを列挙します。
- 配列 (`array: true`) を指定した場合は `max_length` が必須です。
- `endianess` は `little` または `big` を指定できます（省略時は little）。

### 出力結果

- C99: `generated_c/seridl_generated_messages.h` に `typedef`・`#define`・`static inline` 関数を生成します。
- ドキュメント: `--export_docs` 使用時に `docs/COMMANDS.md` に Markdown ドキュメントを生成します。
