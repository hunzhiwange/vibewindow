## Vapor window

You can run the native version with `cargo run`:

```
cargo run 
cargo build --verbose
cargo clippy --verbose --all-targets
cargo test --verbose
cargo fmt -- --check
cargo doc
```

## CLI

- `vw_server`: 本地 HTTP API server（Axum），文档见 [src/bin/vw_server.md](file:///Users/xiongjiaojiao/code/vibe-window2/src/bin/vw_server.md)
- `vw_mcp`: MCP CLI，文档见 [src/bin/vw_mcp.md](file:///Users/xiongjiaojiao/code/vibe-window2/src/bin/vw_mcp.md)
- `vibe-agent`: Vibe Window AI Agent，文档见 [docs/vibe-agent-usage.md](docs/vibe-agent-usage.md)

The web version can be run with [`trunk`]:

```
rustup target add wasm32-unknown-unknown
cargo install trunk
cd crates/vw-desktop
trunk serve ../../index.html
```

`trunk` 需要在 `vw-desktop` 包上下文中运行，目标 HTML 使用仓库根目录的 `index.html`。

[`trunk`]: https://trunkrs.dev/

## release

Run the bundling script to create a macOS application bundle containing both the main app and the webview helper:

```bash
./scripts/bundle_macos.sh
```
