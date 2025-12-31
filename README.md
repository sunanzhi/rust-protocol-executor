# Rust Protocol Executor

一个可扩展的协议执行器，当前内置支持 HTTP 与 WebSocket，并通过中间件链进行前置/后置处理。支持以可执行文件形式的外部中间件（stdin/stdout JSON 协议），可按协议组合使用。

**核心特性**
- 协议执行器：HTTP、WebSocket（可扩展到 TCP/UDP 等）
- 中间件体系：内置 logger/validator/metrics，中间件有序执行
- 外部中间件：以独立进程运行，通过标准输入输出交换 JSON
- 配置与 CLI：支持 YAML 配置文件和命令行参数
- 异步执行：基于 Tokio、tracing 日志

**项目结构**
- [src/main.rs](file:///d:/projects/rust-protocol-executor/src/main.rs)：CLI 入口，解析参数、加载配置、注册中间件并执行协议
- [src/cli/mod.rs](file:///d:/projects/rust-protocol-executor/src/cli/mod.rs)：命令行参数定义（协议、目标、Headers、Payload、超时、重试、外部中间件路径等）
- [src/executor](file:///d:/projects/rust-protocol-executor/src/executor)：执行器接口与实现
  - [base.rs](file:///d:/projects/rust-protocol-executor/src/executor/base.rs)：中间件链执行框架
  - [http.rs](file:///d:/projects/rust-protocol-executor/src/executor/http.rs)：HTTP 执行器
  - [websocket.rs](file:///d:/projects/rust-protocol-executor/src/executor/websocket.rs)：WebSocket 执行器
- [src/middleware](file:///d:/projects/rust-protocol-executor/src/middleware)：中间件体系
  - [traits.rs](file:///d:/projects/rust-protocol-executor/src/middleware/traits.rs)：中间件接口
  - [registry.rs](file:///d:/projects/rust-protocol-executor/src/middleware/registry.rs)：中间件注册与发现
  - [builtin](file:///d:/projects/rust-protocol-executor/src/middleware/builtin)：内置日志/校验/指标
  - [external.rs](file:///d:/projects/rust-protocol-executor/src/middleware/external.rs)：外部中间件通信协议与封装
- [examples/config.yaml](file:///d:/projects/rust-protocol-executor/examples/config.yaml)：示例配置（注意：旧格式示例，见下方“配置文件”部分的最新格式）
- [middlewares/response_formatter](file:///d:/projects/rust-protocol-executor/middlewares/response_formatter)：外部中间件示例（响应格式化器）

**环境要求**
- Rust（建议 1.75+），Edition 2024
- Windows/Linux/macOS
- 访问网络时需允许出站连接

**快速开始**
- 构建

```bash
cargo build
```

- 运行（HTTP 示例）

```bash
cargo run -- http https://httpbin.org/get -H "Accept=application/json" --timeout 30 -r 1 -v
```

- 运行（WebSocket 示例）

```bash
cargo run -- ws wss://echo.websocket.events -p "hello from executor" -v
```

命令行参数来源：[cli](file:///d:/projects/rust-protocol-executor/src/cli/mod.rs)
- protocol：协议（http、https、ws、wss、tcp、udp）
- target：目标地址或 URL
- -p/--payload：请求体（HTTP 原样作为 body，WS 作为文本消息）
- -H/--headers key=value（可多次）
- --timeout 秒、-r/--retry-count 次
- -m/--middleware 指定外部中间件可执行文件路径（可多次）
- --dry-run 仅执行中间件不发起协议请求（预留）
- -v/--verbose 输出详细信息

**配置文件（最新格式）**
加载方式：通过 -c/--config 指定 YAML 文件路径，或默认使用内置默认值。
对应结构：[config](file:///d:/projects/rust-protocol-executor/src/config/mod.rs)

示例（与当前实现匹配的简化格式）：

```yaml
default_timeout: 30
default_retry_count: 2
external_middleware_dir: "./middlewares"

middleware_chains:
  http:
    - name: logger
      type:
        Builtin: "logger"
      enabled: true
      order: 1
    - name: validator
      type:
        Builtin: "validator"
      enabled: true
      order: 2
    - name: response_formatter
      type:
        External: "middlewares/response_formatter/response_formatter.exe"
      enabled: true
      order: 100
      config: {}

protocols:
  http:
    default_headers:
      User-Agent: "cli-executor/1.0"
      Accept: "application/json"
```

说明：
- middleware_chains.<protocol> 是一个数组，按 order 升序依次执行
- type 为枚举，需要以 YAML 方式表示为对象键：Builtin/External
- External 的值为中间件可执行文件路径（Windows 支持 .exe/.bat/.cmd/.ps1；Linux/macOS 需具备执行权限）
- protocols 可为不同协议预设默认 headers/选项
- 如果仅通过 external_middleware_dir 目录注册外部中间件，也可以不在 middleware_chains 中显式写入路径，改为在 CLI 用 -m 追加

注意：仓库内的 [examples/config.yaml](file:///d:/projects/rust-protocol-executor/examples/config.yaml) 展示了较早期结构，仅供参考，实际以上述“最新格式”为准。

**外部中间件协议**
- 输入：JSON（经 stdin 传入），字段见 [ExternalMiddlewareRequest/Response](file:///d:/projects/rust-protocol-executor/src/middleware/external.rs#L12-L35)
- 输出：JSON（经 stdout 返回），结构需与 Response 匹配
- 失败：非 0 退出码或 should_continue=false 将阻断执行

快速体验示例（响应格式化器）：
- 位置：[response_formatter](file:///d:/projects/rust-protocol-executor/middlewares/response_formatter)
- 用法（Windows）：

```bash
echo "{\"action\":\"after\",\"execution_id\":\"id\",\"protocol\":\"http\",\"target\":\"/v1/user\",\"headers\":[],\"metadata\":[],\"timestamp\":\"\"}" | .\\middlewares\\response_formatter\\target\\release\\response_formatter.exe
```

关于实现可参考：
- 中间件接口：[traits.rs](file:///d:/projects/rust-protocol-executor/src/middleware/traits.rs)
- 外部中间件封装与超时/错误处理：[external.rs](file:///d:/projects/rust-protocol-executor/src/middleware/external.rs)
- 内置中间件：日志 [logger.rs](file:///d:/projects/rust-protocol-executor/src/middleware/builtin/logger.rs)、校验 [validator.rs](file:///d:/projects/rust-protocol-executor/src/middleware/builtin/validator.rs)、指标 [metrics.rs](file:///d:/projects/rust-protocol-executor/src/middleware/builtin/metrics.rs)

**扩展指南**
- 新增协议
  - 定义结构体并实现 [Executor](file:///d:/projects/rust-protocol-executor/src/executor/mod.rs#L37-L44) trait
  - 在 [ExecutorFactory](file:///d:/projects/rust-protocol-executor/src/executor/mod.rs#L53-L117) 中匹配并返回该执行器
- 新增内置中间件
  - 实现 [Middleware](file:///d:/projects/rust-protocol-executor/src/middleware/traits.rs#L6-L23) trait
  - 在 [registry.rs](file:///d:/projects/rust-protocol-executor/src/middleware/registry.rs#L44-L64) 注册
- 新增外部中间件
  - 编写可执行程序，按 stdin/stdout JSON 协议收发
  - 通过 external_middleware_dir 或 CLI -m 参数加载

**调试与日志**
- 使用 tracing_subscriber 初始化日志：[main.rs](file:///d:/projects/rust-protocol-executor/src/main.rs#L15)
- 建议设置 RUST_LOG 环境变量提升可观测性，例如：

```bash
$env:RUST_LOG="info"   # PowerShell
```

**测试**
- 运行单元测试

```bash
cargo test
```

示例测试位于 [tests/example.rs](file:///d:/projects/rust-protocol-executor/tests/example.rs)。

**许可证**
- 本项目采用 [LICENSE](file:///d:/projects/rust-protocol-executor/LICENSE) 中的许可条款
