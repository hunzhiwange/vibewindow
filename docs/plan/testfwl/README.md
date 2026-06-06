# 单元测试覆盖率分析

本目录记录 VibeWindow 当前单元测试覆盖面的静态分析与分批补测计划。

## 口径

- 分析日期：2026-05-31
- 分析方式：静态扫描 Rust 源文件、测试文件、`#[test]`、`#[tokio::test]`、`#[cfg(test)]` 引用关系。
- 未执行：`cargo test`、Makefile 目标、`cargo llvm-cov`。
- 原因：仓库工作协议要求未明确授权时不执行 `cargo test`/Makefile。真实行覆盖率需要后续单独授权运行。

## 文件

- `01-static-baseline.md`：workspace 与 crate 级静态测试面。
- `02-risk-gaps.md`：按风险排序的覆盖缺口。
- `03-batch-plan.md`：分批补测计划与验收口径。

## 总结

当前测试文件数量充足，核心安全、shell、gateway、ACP、配置类型都有测试入口；但覆盖质量不均。`vw-desktop` 存在较多“符号存在性”测试，会抬高测试数量但不等价于行为覆盖。下一步应优先补齐高风险异步流程、权限/网关边界、workflow 调度、UI 状态纯函数与 ACP 队列/持久化失败路径。
