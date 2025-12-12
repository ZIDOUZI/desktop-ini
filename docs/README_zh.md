# desktop-ini

[![Crates.io](https://img.shields.io/crates/v/desktop-ini.svg)](https://crates.io/crates/desktop-ini)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

[English](../README.md) | [简体中文](README_zh.md)

一个用于在 Windows 上管理目录 `desktop.ini` 的小工具。

## 功能概览

- **查看** 指定目录的 `desktop.ini` 并以易读格式展示关键信息。
- **设置** 目录的替代标题、图标、悬停提示、标签（Prop5）、自定义执行命令等字段。
- **执行** `desktop.ini` 中配置的自定义命令，带可选二次确认。
- 如果执行需要管理员权限，会自动触发 UAC 弹窗并尝试提升权限后重试。
- **同步** 当前目录下所有包含 `desktop.ini` 的子目录，将其目录属性设为只读。
- **注册表集成**：在资源管理器中为自定义目录类注册，使用本程序打开。

## 安装

```bash
# 在项目根目录
cargo install --path .
```

安装完成后，可以使用生成的可执行文件名（默认为 `desktop-ini`，取决于你的构建环境）来调用。

## 基本用法

所有子命令都支持以下全局参数：

- `--path <DIR>`：目标目录，默认为当前工作目录。
- `--error-action <MODE>`：错误处理策略，取值：
  - `continue`：打印错误后继续（默认）。
  - `inquire`：出错时停下并等待按回车继续。
  - `silently`：忽略错误，不输出提示。
  - `stop`：遇到错误立即退出。
- `--dry-run`：模拟模式，只打印将要执行的写操作，不真正修改文件/属性。

### 查看目录信息

```bash
desktop-ini show --path <DIR>
```

会以紧凑的摘要形式打印 `desktop.ini` 里的：

- 标题 (`LocalizedResourceName`)
- 悬停提示
- 图标 (`IconResource`)
- 标签（Prop5 解析后的 tag 列表）
- 自定义执行命令及其二次确认状态

### 设置 desktop.ini

```bash
desktop-ini set \
  --path <DIR> \
  --name "示例文件夹" \
  --icon "shell32.dll,4" \
  --info-tip "这是一个示例" \
  --add-tag work --add-tag rust \
  --command "code" \
  --args "%1" \
  --confirm
```

所有选项：

- `--name`：设置目录显示名称 (`LocalizedResourceName`)。
- `--icon`：设置图标 (`IconResource`)，例如 `"shell32.dll,4"`。
- `--info-tip`：悬浮提示 (`InfoTip`)。
- `--add-tag` / `--remove-tag` / `--clear-tag`：管理标签。
  - `--add-tag` / `--remove-tag` 可以传入多次。
  - `--add-tag` / `--remove-tag` 也支持逗号分隔，例如 `--tag a,b,c`。
- `--command`：设置自定义执行命令的可执行文件（写入 `Target`）。
- `--args`：设置自定义执行命令参数（写入 `Args`）。
  - `--args` 可以传入多次。
  - `%1` 会替换为包含 `desktop.ini` 的目录路径。
  - `%%` 表示字面量 `%`。
  - 引号用于将包含空格的内容视为一个参数；引号内支持 `\"` 与 `\\`。
- `--confirm`：启用执行前二次确认；不带该参数且原本已启用时，会关闭确认。

> 使用 `--dry-run` 可以先预览将要写入的 `desktop.ini` 内容。

### 执行 desktop.ini 中的命令

```bash
desktop-ini run --path <DIR>
```

- 若配置了二次确认，会提示：
  - `y/yes`：执行命令。
  - `n/no`：退出。
  - `o/open`：在资源管理器中打开目录。
  - `f/file`：使用资源管理器中打开 `desktop.ini`。
- 若 `desktop.ini` 中未配置 `Target`，则程序将直接退出。
- `Args` 采用“空白分隔 + 引号分组”的规则解析，并支持 `%1`/`%%` 展开。
- 当 Windows 返回错误码 `740`（需要提升权限）时，会触发 UAC 并通过 `ShellExecuteW` 重试。

### 批量同步只读属性

```bash
desktop-ini sync --path <ROOT> --depth <N>
```

- 从 `ROOT` 开始递归遍历，找到包含 `desktop.ini` 的目录，将目录属性设为只读。
- `--depth` 控制递归深度，省略时视为无限深度。
- 搭配 `--dry-run` 可以预览会影响的目录而不进行修改。

### 注册表集成

```bash
desktop-ini setup
```

在当前用户的注册表下创建自定义目录类：

- `HKCU\Software\Classes\INI.CustomExecution`（由 `DIRECTORY_CLASS` 常量控制）。
- 将其 `Shell\open\command` 指向当前可执行文件的 `run` 子命令。

之后可以在 `desktop.ini` 中通过 `DirectoryClass=INI.CustomExecution` 和 `[.CustomExecution]` 来让资源管理器调用本程序处理打开行为。

### Shell 补全脚本

```bash
desktop-ini completion | Out-String | Invoke-Expression
```

上述命令会生成 PowerShell 补全脚本并加载到当前 PowerShell 会话中。写入 `$PROFILE` 以永久生效。

## 开发与测试

```bash
# 运行测试
cargo test

# 检查格式与编译（如有配置）
cargo build
```

本项目主要依赖：

- `clap` / `clap_complete`：命令行解析与补全
- `owo-colors`：彩色终端输出
- `encoding_rs`：按系统 ANSI 代码页读写文本
- `thiserror`：错误类型定义
- `winreg`：Windows 注册表操作
- `windows-sys`：Windows API 调用
