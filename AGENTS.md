# AGENTS

## 项目当前状态

- 项目名：`ssh-bench`
- 当前阶段：实现中
- 已完成：
  - 需求设计文档已写入 `docs/superpowers/specs/2026-05-02-ssh-bench-design.md`
  - 过程性实施文档已写入 `.tmp/implementation-context.md`
  - 过程性实施计划已写入 `.tmp/2026-05-02-ssh-bench-implementation-plan.md`
- 正在进行：
  - 进行最终静态检查与收尾提交
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
  - `src/bench/`：四类 benchmark runner 并发执行与分发入口
  - `tests/report_render.rs`：报告渲染测试
  - `src/model.rs`：延迟模式按 warmup/parallel/number 裁剪成功样本后再统计
  - `tests/cli_config.rs`：补充 CLI 互斥/边界/size 取整测试
  - `src/error.rs`：稳定错误类别枚举 `ErrorKind`
  - `src/report.rs`：增加错误分布与成功速率输出
  - `src/target.rs`：支持 host-list 行裁剪空白
  - `src/report.rs`：throughput 汇总速率字段命名改为 aggregate rate
  - `src/bench/throughput.rs`：吞吐率按读取阶段计时，不计入连接建立时间
  - `src/error.rs`：细化到认证/session/exec/命令超时/读取超时等错误类别
  - `src/ssh/auth.rs`：区分私钥文件不存在与私钥格式错误
  - `src/model.rs`：延迟模式按 warmup/parallel/number 裁剪原始样本窗口后再过滤成功样本统计
  - `src/ssh/client.rs`：认证阶段错误统一为 authentication 前缀分类
  - `src/model.rs`：throughput 报告新增 setup latency 聚合字段
  - `src/ssh/session.rs`：throughput setup latency 覆盖 session open + exec 请求阶段
  - `src/ssh/client.rs`：连接阶段错误映射细化到 handshake/tcp connect
  - `src/ssh/session.rs`：command 超时场景正确标记 missing exit status
  - `src/report.rs`：throughput 分位数文本输出改为 bytes_per_ms 单位标签
  - `src/cli.rs`：throughput 模式归一化为单轮执行，不使用 warmup/number
  - `src/ssh/session.rs`：throughput 错误路径正确传递 missing exit status
  - `src/report.rs`：文本报告改为人类可读单位输出，并将错误类别稳定渲染为 snake_case
  - `tests/report_render.rs`：补充文本报告单位缩放与裁剪窗口行为测试
  - `docs/superpowers/specs/2026-05-02-ssh-bench-design.md`：补充 connect-list 空白行处理与文本报告展示约定
  - `tests/cli_config.rs`：补充 identity 必填、runner 选择、默认 throughput 模板与模板占位符校验测试
  - `src/model.rs`：延迟模式成功/失败计数与错误分布改为基于裁剪后的 measured window 聚合
  - `tests/report_render.rs`：补充延迟模式裁剪窗口计数口径回归测试
  - `src/ssh/client.rs`：新增 `ClientProfile` 与 `connect_authenticated_with_profile`，throughput profile 经实测回退为默认 `russh` 配置
  - `src/bench/throughput.rs`：throughput runner 接入 `ClientProfile::Throughput`，disconnect 改为 best-effort 以避免清理失败覆盖已采集结果

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
