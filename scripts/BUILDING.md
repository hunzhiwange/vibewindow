# 构建与打包（macOS / Windows）

本文档放在 `scripts/` 下，面向日常开发与发布。

## 0. 通用前置

- 安装 Rust（推荐用 rustup）：
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- 进入项目根目录：
  ```bash
  cd /path/to/vibe-window
  ```

## 1. macOS（本机构建 + .app 打包）

### 1.1 本机构建（二进制）
```bash
cargo build
cargo test
```

Release：
```bash
cargo build --release
```

### 1.2 macOS .app 打包（包含 vw-webview helper）

安装 `cargo-bundle`（提供 `cargo bundle` 命令）：
```bash
cargo install cargo-bundle
```

执行打包脚本：
```bash
./scripts/bundle_macos.sh
```

产物默认在：
- `target/release/bundle/osx/VibeWindow.app`

## 2. Windows（在 Windows 机器上本机构建）

### 2.1 工具安装

- 安装 Rust（rustup）：https://rustup.rs/
- 安装 MSVC 编译工具链（任选其一）：
  - Visual Studio 2022（勾选 “Desktop development with C++”）
  - 或 Build Tools for Visual Studio（同样需要 C++ 工具链）
- WebView 运行时：
  - `vw-webview.exe` 依赖 Microsoft Edge WebView2 Runtime（Win10/11 通常已自带；没有的话安装即可）

### 2.2 构建

```powershell
cargo build --release
```

产物（示例）：
- `target\release\vibe-window.exe`
- `target\release\vw-webview.exe`

分发时确保两个 exe 同目录。

## 3. macOS 上交叉编译 Windows（推荐：MSVC 目标 + cargo-xwin）

### 3.1 安装目标与工具

安装 Windows MSVC target：
```bash
rustup target add x86_64-pc-windows-msvc
```

安装 `cargo-xwin`：
```bash
cargo install cargo-xwin
```

如果构建时提示找不到 `lld-link` / 链接器相关错误，安装 LLVM 并把它放到 PATH：
```bash
brew install llvm
export PATH="$(brew --prefix llvm)/bin:$PATH"
```

### 3.2 交叉编译

```bash
cargo xwin build --release --target x86_64-pc-windows-msvc
```

产物位置：
- `target/x86_64-pc-windows-msvc/release/vibe-window.exe`
- `target/x86_64-pc-windows-msvc/release/vw-webview.exe`

### 3.3 一键打包成 zip

项目内已提供脚本（会优先使用 cargo-xwin；否则退回 cargo build）：
```bash
bash scripts/bundle_windows.sh
```

默认输出：
- `dist/windows/VibeWindow-x86_64-pc-windows-msvc-release.zip`

可选环境变量：
```bash
TARGET=x86_64-pc-windows-msvc PROFILE=release OUT_DIR=dist/windows bash scripts/bundle_windows.sh
```
