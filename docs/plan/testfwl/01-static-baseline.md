# 静态基线

## Workspace 测试面

本表不是运行时覆盖率，而是静态测试面指标：生产 Rust 文件、测试 Rust 文件、生产/测试代码行数、测试函数数量，以及生产模块中用于挂载独立测试文件的 `#[cfg(test)]` 数量。

| crate | 生产文件 | 测试文件 | 生产行数 | 测试行数 | 测试函数 | `cfg(test)` |
|---|---:|---:|---:|---:|---:|---:|
| `vw-desktop` | 872 | 862 | 281512 | 16995 | 1268 | 878 |
| `vw-agent` | 760 | 878 | 224051 | 93085 | 4238 | 765 |
| `vw-acp` | 90 | 100 | 22237 | 8794 | 321 | 69 |
| `vw-cli` | 58 | 55 | 22608 | 3890 | 139 | 57 |
| `vw-shared` | 34 | 31 | 7102 | 748 | 61 | 30 |
| `vw-figma-json` | 69 | 67 | 6977 | 13375 | 613 | 67 |
| `vw-config-types` | 25 | 23 | 5109 | 162 | 25 | 23 |
| `vw-api-types` | 19 | 18 | 4018 | 454 | 22 | 18 |
| `vw-gateway-client` | 20 | 20 | 2998 | 385 | 30 | 20 |
| `vw-provider-resolver` | 10 | 9 | 1617 | 95 | 10 | 9 |
| `vw-webview` | 1 | 1 | 940 | 6 | 1 | 1 |

## 观察

- `vw-agent` 测试函数最多，重点覆盖 shell、gateway、channels、tools、memory、security 等运行时核心。
- `vw-desktop` 文件级测试入口接近生产文件数，但大量测试是 `include_str!` + 符号存在性断言，适合作为迁移护栏，不足以证明 UI 行为正确。
- `vw-acp` 同时有 `src/*_tests.rs` 与 `tests/*` 集成式测试，队列、会话、CLI、公有协议边界测试较集中。
- `vw-config-types`、`vw-api-types`、`vw-provider-resolver` 以序列化、默认值、模型解析等轻量测试为主，适合作为稳定契约层继续保持高覆盖。
- `scripts/unit-coverage.sh` 已提供 `cargo llvm-cov` 入口，可在获得明确授权后生成真实 HTML 覆盖率报告。

## 重点子系统静态面

| 子系统 | 生产行数 | 测试行数 | 测试函数 | 判断 |
|---|---:|---:|---:|---|
| `vw-agent/tools/shell` | 7401 | 2697 | 231 | 高风险区域已有较密测试，应补组合场景与权限回归。 |
| `vw-agent/gateway/api` | 14313 | 1267 | 96 | 入口多、行为面宽，测试行数偏低。 |
| `vw-agent/session/llm` | 5899 | 434 | 33 | 流式、reasoning、tool call、错误映射仍偏薄。 |
| `vw-agent/workflow` | 约 900 | 低 | 低 | 调度、条件、变量、失败路径需要优先补。 |
| `vw-desktop/app/components` | 73569 | 5182 | 375 | UI 组件面大，很多测试偏结构存在性。 |
| `vw-desktop/app/views` | 68916 | 4390 | 309 | 设计/画布/视图状态测试不足。 |
| `vw-acp/session_runtime` | 约 3500 | 约 300 | 13+ | 生命周期有测试，连接/加载/失败路径仍需加强。 |

## 真实覆盖率入口

后续如需得到真实行覆盖率，可在明确授权后执行：

```bash
./scripts/unit-coverage.sh --output-dir coverage/workspace
./scripts/unit-coverage.sh --package vw-agent --output-dir coverage/vw-agent
```

该脚本会调用 `cargo llvm-cov --workspace --all-features --lib --bins` 或按 package 运行，并写出 HTML 报告。
