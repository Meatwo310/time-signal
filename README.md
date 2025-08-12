# Time Signal

## WIP
このプロジェクトは現在開発中です

TODO:
- カスタマイズ性を何とかする
- 自動ビルドを設定する
- まともなドキュメントを書く(半分Claudeに書かせましたごめんなさい)
  - 英語か日本語か統一する


## 概要
VOICEVOXを使用した音声時報システムです。15分おきに時報を自動で再生します。

このプロジェクトは以下の2つの主要機能を提供します:
1. **音声ファイル生成**: VOICEVOXの音声合成APIを使用して、24時間分(15分間隔)の時報音声ファイルを生成
2. **自動時報**: 現在時刻に対応する時報を15分間隔で自動再生


## 必要な環境
- Windowsマシン (他のOSでも動くとは思いますが未検証です)
- Rust / Cargo
- [VOICEVOX](https://github.com/VOICEVOX/voicevox) か [エンジン](https://github.com/VOICEVOX/voicevox_engine) (デフォルトで `http://127.0.0.1:50021/` へ接続します)


## ビルド
```terminal
git clone https://github.com/Meatwo310/time-signal.git --depth 1
cd time-signal
cargo build --release
```

`./target/release/time-signal.exe`に実行ファイルが生成されます。


## 使用方法

```terminal
Usage: time-signal.exe [COMMAND]

Commands:
  gen   
  run   
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
```terminal
Usage: time-signal.exe gen [OPTIONS] [SPEAKER_ID]

Arguments:
  [SPEAKER_ID]  The speaker ID to use for the voice generation. Leave empty to list all speakers

Options:
  -u, --url <URL>  The URL of the VOICEVOX server [default: http://127.0.0.1:50021/]
  -h, --help       Print help
```

### 1. 利用可能な話者の確認
```terminal
time-signal.exe gen
```
利用可能な話者とスタイルの一覧が表示されます。

### 2. 音声ファイルの生成
```terminal
time-signal.exe gen <speaker_id>
```

例: `VOICEVOX:四国めたん(ノーマル)` の時報を生成:
```terminal
time-signal.exe gen 2
```

このコマンドにより、`voice_files/` ディレクトリに以下のような音声ファイルが計96個(合わせて8MB程度)生成されます:
- `00-00.wav` (0時です)
- `00-15.wav` (0時15分です)
- `00-30.wav` (0時30分です)
- `00-45.wav` (0時45分です)
- ...

> [!NOTE]
> 音声ファイルの生成にはVOICEVOXまたはエンジンを起動しておく必要があります。

### 3. 時報の実行
```terminal
time-signal.exe
```
このコマンドを実行している間、15分間隔(毎時0分、15分、30分、45分)で対応する時報が再生されます。

> [!NOTE]
> runモードは事前生成された音声ファイルを使用します。VOICEVOXは不要です。


## ライセンス
このプロジェクトは[MITライセンス](LICENSE)の下で公開されています。


## VOICEVOX について
このソフトウェアは[VOICEVOX](https://voicevox.hiroshiba.jp/)を使用しています。
音声の生成・利用の際には、VOICEVOX及び各音声ライブラリの規約に従ってください。

