# Time Signal

[![License](https://img.shields.io/badge/License-Apache_2.0_OR_MIT-blue.svg)](#ライセンス) [![Test and Build](https://github.com/Meatwo310/time-signal/actions/workflows/rust.yml/badge.svg)](https://github.com/Meatwo310/time-signal/actions/workflows/rust.yml)

## WIP
このプロジェクトは現在開発中です

TODO:
- [ ] カスタマイズ性を何とかする
- [x] 自動ビルドを設定する
- [ ] まともなドキュメントを書く(半分Claudeに書かせましたごめんなさい)
  - [x] 英語か日本語か統一する
- [x] システムトレイへ格納できるようにする
- [ ] CIでクロスビルド機能を使う？


## 概要
VOICEVOXを使用した音声時報システムです。15分おきに時報を自動で再生します。

このプロジェクトは以下の2つの主要機能を提供します:
1. **音声ファイル生成**: VOICEVOXの音声合成APIを使用して、24時間分の時報音声ファイルを生成
2. **時報システム**: 現在時刻に対応する時報を自動再生


## 必要な環境
- x64のWindows/Linuxマシン、またはarm64のmacOSマシン
  - Windows以外は未検証です
- Rust >=1.85.1 / Cargo
- [VOICEVOX](https://github.com/VOICEVOX/voicevox) または [エンジン](https://github.com/VOICEVOX/voicevox_engine) (デフォルトで `http://127.0.0.1:50021/` へ接続します)


## ダウンロード
Windows / Linux / macOS(arm64) 向けのバイナリが [Releases](https://github.com/Meatwo310/time-signal/releases) に転がっています。
環境が合わない場合は自前でビルドしてください。

実行ファイルは単独で機能します。
音声ファイルはカレントディレクトリ上の `voice_files/` ディレクトリに保存されますので、あんまり変な場所に置くと後で困ります。

## ビルド
```terminal
git clone https://github.com/Meatwo310/time-signal.git --depth 1
cd time-signal
cargo build --release
```

`./target/release/time-signal.exe`に実行ファイルが生成されます。


## 使い方

```terminal
Usage: time-signal.exe [COMMAND]

Commands:
  gen   音声ファイルを事前生成します。VOICEVOXサーバーが必要です。
  run   一定間隔で時報を再生します。音声ファイルを事前に生成する必要があります。
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
```terminal
音声ファイルを事前生成します。VOICEVOXサーバーが必要です。

Usage: time-signal.exe gen [OPTIONS] [SPEAKER_ID]

Arguments:
  [SPEAKER_ID]  音声生成に使用するスタイルID。 空の場合はすべてのスタイルを一覧表示します

Options:
  -u, --url <URL>            VOICEVOXサーバーのURL [default: http://127.0.0.1:50021/]
  -i, --interval <INTERVAL>  時報の間隔。15分を指定すると、毎時0分、15分、30分、45分に音声が生成されます。 [default: 15]
  -h, --help                 Print help
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
> genコマンド実行中は、VOICEVOXまたはエンジンを起動しておく必要があります。

### 3. 時報システムを開始
```terminal
time-signal.exe
```
このコマンドを実行している間、15分間隔(毎時0分、15分、30分、45分)で対応する時報が再生されます。

> [!NOTE]
> runコマンドは事前生成された音声ファイルを使用します。VOICEVOXは不要です。

### ヒント: システムトレイに格納するには
`--cli`オプションが与えられない限り、runコマンドはシステムトレイアイコンを作成します。

Windowsでターミナルウィンドウを表示させずに起動するには、[`no-terminal.ps1`](no-terminal.ps1)を使用してください。

### ヒント: 時報の間隔を調整するには
`-i`/`--interval` オプションを使用して、時報の間隔を変更できます。

例えば、30分ごとに時報を再生するには:
```terminal
time-signal.exe run --interval 30
```

10分ごとに時報を再生するには:
```terminal
time-signal.exe gen --interval 10 <speaker_id>
time-signal.exe run --interval 10
```

> [!NOTE]
> 時報の間隔を短くする際には、音声ファイルの再生成が必要です。
> ファイルが不足している場合は、runコマンド実行時に警告が表示されます。


## ライセンス
このプロジェクトは、**[Apache License 2.0](LICENSE-APACHE)** と **[MIT License](LICENSE-MIT)** のデュアルライセンスです。

サードパーティライセンスについては、各リリースのアーカイブに含まれる `LICENSES.md` ファイルを参照してください。


## VOICEVOX について
このソフトウェアは[VOICEVOX](https://voicevox.hiroshiba.jp/)を使用しています。
音声の生成・利用の際には、VOICEVOX及び各音声ライブラリの規約に従ってください。

