# ssh-bench 设计文档

## 背景与目标

`ssh-bench` 是一个面向 SSH 链路性能评估的命令行工具，目标是在不同并发度下测量连接建立、session 打开、命令执行和数据读取的延迟与吞吐能力。典型使用场景是测试 SSH gateway 或普通 sshd 在不同并发下的 p50、p99 延迟以及整体吞吐量，用于后续人工对比不同链路路径的性能差异。

本工具只负责测量，不负责解释 gateway 与非 gateway 的误差来源，也不负责对比结果的业务含义计算。

## 原始需求整理

### 技术选型

- 使用 Rust 开发
- 使用 `tokio` 进行异步并发执行
- 使用 `russh` 建立 SSH 客户端连接并完成认证、session 与命令执行
- 使用 `clap` 解析 CLI 参数

### CLI 需求

预期参数如下：

- `-p, --parallel <number>`：并发数
- `-n, --number <number>`：至少运行的有效测试数；对 throughput 模式无效
- `-w, --warmup <number>`：warmup 数量，计算结果时丢弃前 n 个测试结果
- `-t, --type <auth|session|command|throughput>`：测试类型
- `-c, --connect <host>`：连接目标，格式固定为 `user@host:port`，其中 user、host、port 都不允许省略；host 仅支持主机名或 IPv4，不支持 IPv6
- `-C, --connect-list <path>`：主机列表文件，每行一个 `user@host:port`
- `-i, --identity <path>`：私钥文件路径，必须显式指定
- `--command <command>`：`command` 模式在远端执行的命令
- `--size <size>`：单次 throughput 读取的大小限制，默认 `1GiB`
- `--file <path>`：throughput 模式读取的远端文件，默认 `/dev/zero`
- `--json`：以 JSON 输出报告

### 输出报告需求

#### 延迟类测试

对 `auth`、`session`、`command` 三类测试，至少输出：

- p50
- p99
- max
- min
- avg
- 总成功数
- wall clock
- 平均成功建立速度

统计对象仅包含成功样本；失败样本参与失败计数，但不参与延迟分位数计算。

#### 吞吐类测试

对 `throughput` 测试，至少输出：

- p50
- p99
- max
- min
- avg
- 总传输字节数
- wall clock
- 平均速率
- aggregate throughput

JSON 输出建议固定使用：

- 时间：毫秒数值
- 速率：bytes / millisecond

人类可读输出再进行单位缩放。

### 各测试类型定义

#### auth

每个样本建立一条新的 SSH 连接，计时到认证成功。

- 失败样本记录错误类别
- 失败样本不参与 latency 分位数
- 失败样本参与 `failure_count`

#### session

每个样本建立一条新的 SSH 连接，认证成功后打开 session channel，计时到 channel open confirmed。

根据设计输入，`russh` 的 `channel_open_session().await` 会发送 open request 并等待确认后返回，因此适合作为 session-open 完成点。

#### command

每个样本建立新连接，完成认证、打开 session、发起 exec，计时到远端命令完成。

完成判定规则：

1. 优先使用 `exit-status`
2. 若服务端不返回 `exit-status`，则使用 `EOF/close + timeout` 兜底
3. 报告中标记样本或汇总中存在缺失 `exit-status` 的情况

#### throughput

启动 `parallel` 条连接；每条连接依次完成认证、打开 session、执行读取命令，然后读取 stdout，直到满足以下任一条件：

- 达到 `--size`
- 收到 EOF
- 发生错误
- 超时

每连接吞吐量定义为：

`实际读取字节 / 读取阶段耗时`

额外输出：

- aggregate throughput = 所有成功读取字节 / 整体 wall-clock 耗时
- 连接建立时间不计入 per-connection throughput
- 连接建立时间可作为 setup latency 或 total elapsed 单独报告

### 认证与安全边界

- 工具不验证 SSH host key
- 仅支持 public key 认证
- 私钥不能有 passphrase，即必须完全非交互式
- 默认假设本地与远端均为 Linux/Unix

### connect-list 轮询规则

`connect-list` 用于对多个后端 sshd 做负载均衡采样。

若主机列表长度为 `len`，并发为 `p`，则第 `i` 个并发 worker 的第 `j` 次测试目标为：

`conn_list[(j + i) % len]`

### 样本数量裁剪规则

总测试数量计算方式：

`total = parallel + number + warmup`

计算结果时：

1. 先丢弃前 `warmup` 个结果
2. 再丢弃其后的 `parallel` 个结果
3. 保留中间的 `number` 个结果

最终统计延迟、吞吐量、wall clock 等均基于保留下来的结果集合。

## 补充澄清结论

在需求沟通中进一步确认如下：

1. 认证方式固定为显式私钥文件输入，因此 CLI 增加 `-i, --identity <path>`。
2. `throughput` 模式的远端命令使用占位符模板。
3. 默认 throughput 模板为基于 `dd if=` 的读取命令。
4. 模板至少注入两个变量：
   - `{file}`：远端文件路径
   - `{count}`：按 MiB 计算得到的读取块数
5. `command` 模式的完成判定采用“`exit-status` 优先，缺失时 `EOF/close + timeout` 兜底”的策略。

## 方案选择

最终采用 **方案 A：单二进制、模块化分层实现**。

原因：

- 当前项目为空仓，适合从清晰的模块边界起步
- 四类 benchmark 可以复用 target 解析、SSH 连接封装、统计和报告层
- 该方案比高度抽象的通用 pipeline 更克制，能避免过度设计
- 该方案又比拆阶段只先做一半功能更贴近当前完整交付目标

## 架构设计

建议采用单二进制 crate，按职责划分模块。

### 模块划分

- `src/main.rs`
  - 程序入口
  - 调用 CLI 解析与主执行流程

- `src/cli.rs`
  - `clap` 参数定义
  - 原始参数解析
  - 参数边界校验与归一化

- `src/model.rs`
  - 核心领域模型
  - 如 benchmark 类型、目标主机、样本结果、聚合报告、错误类别

- `src/target.rs`
  - 解析 `user@host:port`
  - 读取 `connect-list`
  - 生成基于 `(j + i) % len` 的目标分配逻辑

- `src/ssh/`
  - `mod.rs`：公共导出
  - `client.rs`：SSH 客户端建立与配置
  - `auth.rs`：私钥加载与认证逻辑
  - `session.rs`：session 打开、exec、stdout 读取等通用封装

- `src/bench/`
  - `mod.rs`
  - `auth.rs`
  - `session.rs`
  - `command.rs`
  - `throughput.rs`
  - 负责四类 benchmark 的单样本执行逻辑与并发调度入口

- `src/stats.rs`
  - 样本裁剪规则实现
  - p50/p99/min/max/avg 统计
  - aggregate throughput 统计

- `src/report.rs`
  - 文本人类可读输出
  - JSON 结构化输出
  - 单位缩放策略

- `src/error.rs`
  - 统一错误类型
  - 将 IO、认证、协议、超时、远端执行等错误映射为稳定类别

### 设计原则

- 网络 IO 与统计逻辑分离
- 参数解析、目标选择、SSH 会话操作、统计汇总、输出格式化分别建模
- 尽量把纯逻辑写成纯函数，降低测试难度
- 将 russh 的状态机复杂性限制在 `ssh/` 和各 `bench/*` runner 边界内
- 对外输出稳定的数据模型，避免报告层直接依赖 russh 内部事件细节

## 数据模型设计

### CLI 归一化配置

在参数解析后，立即归一化为内部配置对象，例如：

- `BenchmarkKind`
- `Parallelism`
- `SamplePlan`
- `TargetSource`
- `IdentityPath`
- `OutputFormat`
- `ThroughputCommandTemplate`

这样后续执行层不再关心 clap 原始值与字符串细节。

### 目标模型

目标主机需要解析为结构化对象：

- `user`
- `host`
- `port`

并在解析边界做如下限制：

- 拒绝缺失 user/host/port
- 拒绝 IPv6 字面量
- 非法格式行应报错
- `connect-list` 读取时允许并忽略纯空白行；非空白内容仍需严格满足 `user@host:port`

### 样本结果模型

每次样本执行产生结构化结果：

- benchmark 类型
- 目标主机
- success / failure
- 错误类别
- 测量值（latency 或 throughput）
- wall time / setup time / read time 等辅助字段
- 是否缺失 exit-status
- 实际读取字节数

这样统计层只需要处理稳定的领域对象，而不依赖运行时临时状态。

### 报告模型

报告输出分为两层：

1. **内部聚合模型**：统一使用毫秒与 bytes/ms
2. **展示层模型**：文本输出时再做单位格式化

该设计可以避免 JSON 与文本输出使用两套不一致的统计口径。

当前实现约定：

- JSON 继续输出毫秒与 bytes/ms 原始数值
- 文本报告对 latency 使用 `ms/s/min` 缩放
- 文本报告对字节与速率使用 `KiB/MiB/GiB` 与 `/s` 展示
- 文本错误分布使用稳定的 `snake_case` 错误类别名称

## 执行流程设计

### 通用主流程

1. 解析 CLI
2. 校验参数组合合法性
3. 加载私钥
4. 加载单目标或目标列表
5. 构建 benchmark 配置
6. 按 benchmark 类型进入相应 runner
7. 收集样本结果
8. 按规则裁剪样本
9. 聚合统计
10. 输出文本或 JSON 报告

### 延迟类 benchmark 流程

对于 `auth`、`session`、`command`，共享一个“生产样本直到达到计划数量”的总体框架：

- 以 `parallel` 个 worker 并发运行
- 每个 worker 按自己的索引和已完成样本序号选择目标主机
- 总共产生 `parallel + number + warmup` 个样本结果
- 样本完成后统一收集并裁剪

### auth 测量边界

- 起点：发起 SSH 连接前或连接动作刚开始前
- 终点：公钥认证成功

不包含后续 session 打开与命令执行时间。

### session 测量边界

- 起点：发起 SSH 连接前或连接动作刚开始前
- 终点：`channel_open_session().await` 成功返回

### command 测量边界

- 起点：发起 SSH 连接前或连接动作刚开始前
- 终点：远端命令被判定完成

命令完成判定：

- 首选 `exit-status`
- 若未收到，则等待 `EOF/close`
- 若仍不完整，则由超时策略终止并记录状态

### throughput 流程

`throughput` 模式与延迟模式不同，重点是固定并发连接下的持续读取而非离散样本数量，因此：

- 启动 `parallel` 条连接
- 每条连接独立建立、认证、打开 session、执行 throughput 命令
- 进入 stdout 读取循环
- 读取直到达到 size、EOF、错误或超时
- 每条连接形成一个吞吐样本

如果 `throughput` 模式仍需遵守 warmup/number 裁剪规则，则实现时需要明确：

- 是将每条连接视为一个样本批次
- 还是让 worker 反复建立多轮 throughput 样本

基于当前需求文字“`-n` 对 throughput 模式无效”，推荐解释为：

- throughput 不使用 `number`
- 仅执行一轮 `parallel` 条连接并输出结果

该点已足够形成实现方向。

## Throughput 命令模板设计

### 模板接口

增加 throughput 专用命令模板参数，例如：

- `--throughput-command <template>`

默认值为基于 `dd if=` 的命令模板，例如：

`dd if={file} bs=1M count={count}`

### 占位符规则

至少支持：

- `{file}`：替换为远端路径
- `{count}`：替换为按 MiB 计算得到的块数

例如：

- `--file /dev/zero --size 1GiB` -> `count = 1024`

### 大小处理策略

由于默认命令以 `bs=1M` 读取，因此 `{count}` 按 MiB 计算。

实现层需要明确以下行为：

- 若 `size` 不是 1 MiB 的整数倍，采用向上取整还是向下取整
- 实际统计时必须以真实读取到的字节数为准，而不是估算值

推荐：

- 命令模板层使用向上取整保证读取上限足够
- 读取循环层继续按 `size` 做本地截断，保证不会超出用户期望的统计上限

## 错误处理设计

### 错误分类

建议将错误稳定映射为如下类别：

- 参数错误
- 目标解析错误
- 私钥读取错误
- 私钥格式错误
- TCP 连接错误
- SSH 握手错误
- 认证失败
- Session 打开失败
- Exec 请求失败
- 远端命令超时
- 远端读取超时
- 非预期协议错误

### 错误统计原则

- 失败样本记录错误类别
- 失败样本进入 `failure_count`
- 失败样本不进入成功样本统计分位数
- 报告中最好能输出错误类别分布，方便判断瓶颈位置

## 测试策略

### 单元测试优先范围

优先为纯逻辑写单元测试：

- `user@host:port` 解析
- host list 读取与校验
- 轮询目标分配
- 样本裁剪逻辑
- 分位数与平均值计算
- throughput 模板渲染
- size 到 `{count}` 的转换

### 集成测试范围

由于 russh 与真实 sshd 交互成本较高，第一阶段集成测试可聚焦：

- 与本地或容器化 sshd 的最小联通验证
- auth / session / command 的基本 happy path
- throughput 的基本读取路径

### 非目标

首版不追求完整的复杂网络故障仿真；先确保：

- 参数行为正确
- 统计口径正确
- 在标准 Linux/Unix sshd 环境下可运行

## 已知限制与显式边界

- 不支持 IPv6 目标地址
- 不验证 host key
- 不支持交互式密码或带 passphrase 的私钥
- 不负责对 gateway 与非 gateway 的差值做业务解读
- throughput 模式默认依赖远端具备 `dd` 等常见 Unix 工具，若用户自定义模板则由用户保证可执行性

## 后续实施建议

建议按以下顺序实现：

1. CLI 与配置归一化
2. target 解析与样本规划
3. stats 与 report 纯逻辑
4. SSH 认证与 session 基础封装
5. auth runner
6. session runner
7. command runner
8. throughput runner
9. 集成验证与文档补充

该顺序可以优先完成高可测的纯逻辑，再逐步接入网络行为，降低调试成本。
