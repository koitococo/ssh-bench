# AGENTS

## 项目当前状态

- 项目名：`ssh-bench`
- 当前阶段：实现中
- 已完成：
  - 需求设计文档已写入 `docs/superpowers/specs/2026-05-02-ssh-bench-design.md`
  - 过程性实施文档已写入 `.tmp/implementation-context.md`
  - 过程性实施计划已写入 `.tmp/2026-05-02-ssh-bench-implementation-plan.md`
- 正在进行：
  - 实现 runner 行为细化与参数约束补全
- 已新增：
  - `src/stats.rs`：最小延迟统计与样本窗口裁剪逻辑
  - `tests/target_stats.rs`：统计纯逻辑测试
  - `src/target.rs`：`user@host:port` 解析与轮询选择逻辑
  - `src/cli.rs`：CLI 参数解析与基础配置归一化
  - `tests/cli_config.rs`：CLI 单目标配置测试
  - `src/ssh/session.rs`：throughput 命令模板渲染逻辑
  - `src/error.rs`：统一应用错误类型
  - `src/model.rs`：样本结果与报告模型
  - `src/report.rs`：文本/JSON 报告渲染
  - `src/ssh/auth.rs`：私钥加载逻辑
  - `src/ssh/client.rs`：SSH 连接与公钥认证封装
  - `src/bench/`：四类 benchmark runner 骨架与分发入口
  - `tests/report_render.rs`：报告渲染测试

## 实现约束

- 与用户始终使用中文交流
- 过程性文档优先写入 `.tmp/`
- 每完成一个明确 TODO 后创建一次 git commit
- 避免提交与当前 TODO 无关的文件
- 审慎使用 TDD：优先用于纯函数、算法和关键逻辑

## 当前设计关键点

- 技术栈：Rust / tokio / russh / clap
- 核心 benchmark：`auth` / `session` / `command` / `throughput`
- 认证方式：显式 `--identity <path>` 私钥文件
- throughput 默认模板：`dd if={file} bs=1M count={count}`
- `command` 完成判定：`exit-status` 优先，缺失时 `EOF/close + timeout` 兜底
