# 覆盖任务：vw-agent/src/skill/audit.rs

- 目标源码：`crates/vw-agent/src/skill/audit.rs`
- 报告来源：`coverage/workspace/html/index.html`
- 报告时间：2026-05-25 14:41
- 当前行覆盖率：10.51%
- 当前区域覆盖率：10.65%
- 当前函数覆盖率：17.78%
- 未覆盖行数：366 / 409
- 目标行覆盖率：100%
- 建议测试文件：`crates/vw-agent/src/skill/audit_tests.rs`

## 测试任务

- 阅读目标源码和相邻测试，先确认当前公开行为。
- 为成功路径、边界值、错误路径补单元测试。
- 测试必须放在独立测试文件中，不与逻辑代码混放。
- 不为测试引入无当前调用者的新抽象或配置。

## 验收命令

- `make unit-coverage-package PACKAGE=vw-agent`
