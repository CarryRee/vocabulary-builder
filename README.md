# 词汇标本室

一款基于 Tauri 与 Dioxus 构建的本地桌面背词工具。它帮助你记录遇到的陌生单词及其来源，并按当前熟悉度整理词汇。

## 功能

- 新增单词、来源 URL 和熟悉度状态。
- 将单词标记为“陌生”“了解”或“熟悉”，并按状态筛选。
- 编辑或删除已有词卡。
- 通过默认浏览器可靠打开来源链接，兼容禁止 iframe 嵌入或 WebView 渲染的网站。
- 使用 SQLite 在本机持久化保存数据，重启应用后仍可继续使用。

## 数据与隐私

所有词汇数据仅保存在本机的应用数据目录中，数据库文件名为 `vocabulary.db`。应用不会上传或同步单词、URL 或使用记录。

Windows 下，数据库通常位于：

```text
C:\Users\<你的用户名>\AppData\Roaming\com.carryree.vocabulary-builder\vocabulary.db
```

目录中的 `com.carryree.vocabulary-builder` 来自应用标识；可备份该数据库文件以保留词汇记录。

## 运行日志

应用会将运行日志保存在本机，不会上传。Windows 下的日志文件通常位于：

```text
C:\Users\<你的用户名>\AppData\Local\com.carryree.vocabulary-builder\logs\vocabulary-builder.log
```

日志以本地时区记录 INFO 及以上级别的信息，同时会输出到开发终端。单个日志文件达到 1 MB 时会轮转，历史日志会保留在同一目录中。可用任意文本编辑器打开，或在 PowerShell 中持续查看最新内容：

```powershell
Get-Content "$env:LOCALAPPDATA\com.carryree.vocabulary-builder\logs\vocabulary-builder.log" -Wait
```

为便于排查问题，日志会包含新增、编辑、删除和打开来源时的单词、来源 URL、熟悉度及错误信息。因此请勿将日志文件分享给不受信任的人；删除该目录中的日志不会删除词汇数据库。

## 技术栈

- [Tauri 2](https://v2.tauri.app/)：桌面应用容器与 Rust 命令层
- [Dioxus 0.7](https://dioxuslabs.com/)：Rust/WASM 前端界面
- [SQLite](https://www.sqlite.org/) + `rusqlite`：本地词汇数据存储

## 开发环境

请准备 Rust stable、Tauri 的系统依赖和 Dioxus CLI：

```bash
cargo install dioxus-cli
```

在项目根目录启动桌面开发环境：

```bash
cargo tauri dev
```

该命令会先启动 Dioxus 前端开发服务器，再打开 Tauri 桌面窗口。

## 打包发布

在项目根目录执行以下命令构建发布版本：

```bash
cargo tauri build
```

该命令会先执行 `dx bundle --release` 构建 Dioxus 前端，再编译 Rust/Tauri 后端，并依据 `src-tauri/tauri.conf.json` 生成 Windows 安装包。产物通常位于：

```text
src-tauri/target/release/bundle/
├── nsis/    Windows 安装程序（.exe）
└── msi/     Windows Installer（.msi）
```

## 验证

```bash
cargo fmt --all -- --check
cargo test -p vocabulary-builder
cargo check -p vocabulary-builder-ui
cargo clippy -p vocabulary-builder -- -D warnings
```

## 项目结构

```text
src/                     Dioxus 前端：表单、词卡列表、筛选、分页与本地日志
src-tauri/src/lib.rs     Tauri 命令、应用状态与数据库初始化
src-tauri/src/vocabulary.rs
                         SQLite 仓储、数据模型、输入校验和单元测试
assets/styles.css        词汇卡片索引界面的样式
```
