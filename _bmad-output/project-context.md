---
project_name: 'YIT-GPA-Calculator-Rust'
user_name: 'David'
date: '2026-02-28'
sections_completed: ['technology_stack', 'critical_rules', 'verification_rules']
existing_patterns_found: 10
---

# Project Context for AI Agents

_本文件包含 AI 代理在本项目中实现代码时必须遵循的关键规则与模式。侧重容易遗漏、不显而易见的细节。_

---

## 内容生成与核实规则（最高优先级）

- **时效性**：生成任何内容前必须 100% 核实时效性（当前年份、依赖版本、API 是否在当前版本可用、文档是否仍适用）。不得使用未核实的“当前可能”“若未支持则……”等表述。
- **用法正确性**：涉及语言特性、库 API、配置项时，必须依据官方文档或源码确认用法正确，不得凭印象或假设编写。
- **禁止假设**：禁止写入未经核实的技术判断（例如“工具链可能尚未支持”）。若无法核实，应明确标注“需人工核实”或不予写入。
- **禁止不确定内容**：不得在项目上下文中出现“可能”“也许”“一般”“若……则”等不确定表述，除非明确标注为待用户确认的选项。

---

## Technology Stack & Versions

_以下版本均来自项目根目录 `Cargo.toml`，生成时已按文件内容核对。_

- **Rust**：edition = `"2024"`。Rust 2024 Edition 于 Rust 1.85.0（2025-02-20）稳定，工具链已支持；本项目使用 2024 edition 正确。
- **Web / 运行时**：axum `0.8.4`（features: `["multipart"]`）、tokio `1.46.1`（features: `["full"]`）。
- **模板与资源**：tera `1.20.1`、rust-embed `8.11.0`（嵌入 `templates/` 与 `assets/`）。
- **会话与 Cookie**：tower-sessions `0.15.0`（features: `["memory-store"]`）、tower-cookies `0.11.0`（features:
  `["signed"]`）。
- **网络与解析**：reqwest `0.13.2`（features: `["json", "cookies"]`）、scraper `0.25.0`、base64 `0.22.1`、url `2.5.8`。
- **序列化**：serde `1.0.228`（features: `["derive"]`）、serde_json `1.0.149`、serde_urlencoded `0.7.1`。
- **数值与业务**：rust_decimal `1.40.0`（features: `["serde", "std"]`）、rust_decimal_macros `1.40.0`。
- **错误与工具**：anyhow `1.0.102`、thiserror `2.0.18`。
- **其他**：chrono `0.4.44`、rand `0.10.0`、calamine `0.33.0`、webbrowser `1.1.0`、fake_user_agent `0.2.3`、
  mime_guess `2.0.5`。

---

## Critical Implementation Rules

### 错误与响应

- **领域错误**：在 `src/models.rs` 中用 `thiserror::Error` 定义 `WebScrapingError`、`FileError`、`WebError`
  。新增领域错误时应在此定义，并通过 `#[from]` 或手动实现并入 `WebError`。
- **WebError 与 HTTP**：handler 层返回 `Result<..., WebError>`。`WebError` 已实现 `IntoResponse`，将各变体映射到具体 HTTP
  状态码（如 `LoginFailed` → UNAUTHORIZED，`FileError` → BAD_REQUEST，其余服务端错误 → INTERNAL_SERVER_ERROR）。新增错误变体时需在
  `IntoResponse` 的 `match` 中补充映射。
- **应用入口与底层**：`main.rs` 与部分底层（如 `scraping`）使用 `anyhow::Result`，并用 `.with_context(|| ...)` 添加上下文后再
  `?`。不要在此类位置返回 `WebError`，需在 handler 边界将 `anyhow` 或领域错误转换为 `WebError`（例如
  `map_err(|e| WebError::InternalError(e.to_string()))`）。

### 分层与模块

- **router**（`src/router.rs`）：仅注册路由与 `with_state(tera)`，不包含业务或 HTTP 细节。
- **handler**（`src/handler.rs`）：处理 HTTP（Form/Json/Multipart/Session）、调用 business/scraping、渲染 Tera、返回
  `Result<..., WebError>`。
- **business**（`src/business.rs`）：GPA 计算、成绩转换、排除规则常量（如 `PERMANENT_IGNORED_COURSES`、`ATTR_EXCLUSIONS`、
  `EXCLUDED_COURSES_KEYWORD`）。与 HTTP 无关。
- **scraping**（`src/scraping.rs`）：教务网请求、登录、成绩抓取，返回 `Result<..., WebScrapingError>` 或 `anyhow::Result`。
- **models**（`src/models.rs`）：数据结构（如 `Course`）与所有领域错误类型。内部/仅本 crate 使用的模块以 `_` 前缀命名（如
  `_url`）。

### URL 构建与拼接（硬性规定）

- **必须使用 `url` crate**：凡涉及 URL 的拼接、路径追加、基址+路径组合等，一律使用 `Cargo.toml` 中的 `url` 依赖（rust-url，如
  `url::Url`、`path_segments_mut()`、`join()` 等 API）来构建，不得手写完整 URL 字面量字符串再转换。
- **唯一例外**：仅允许在 `src/_url.rs` 中、为得到 `Url::parse(...)` 的输入而用 `format!("{}://{}/", scheme, host)` 等形式拼出
  **基址字符串**的这一处（即构造 base URL 的那一行）。除此以外，项目内任何其他地方均不得用“先拼字符串再
  parse/from_str”或“format! 整段 URL”等方式构建 URL。
- **禁止的写法**：除上述例外外，不接受“先写整段 URL 字符串再 `parse`/`from_str`”“用 `format!` 拼整串 URL”“在代码里直接写死路径字符串再拼到
  base”等任何不通过 `url` 做路径/查询构造的方式。此类写法易在维护时写错路径、漏斜杠或难以重构，一律禁止。
- **既有约定**：项目中已有通过 `_url` 模块封装 `Url` 与 `push` 等逻辑的用法，新增或修改 URL 相关逻辑时须沿用同一思路，始终用
  `url` 类型与 API 完成拼接。

### 数值与业务常量

- **GPA / 学分 / 绩点**：一律使用 `rust_decimal::Decimal`，字面量使用 `dec!` 宏（如 `dec!(2.33)`）。禁止用 `f32`/`f64`
  做绩点或学分计算。
- **排除与过滤**：课程排除逻辑依赖 `business` 中的 `PERMANENT_IGNORED_COURSES`、`ATTR_EXCLUSIONS`、
  `EXCLUDED_COURSES_KEYWORD`。修改排除规则时只改这些常量或 business 内逻辑，不在 handler/scraping 中硬编码。

### 状态与扩展

- **Tera**：通过 `State<Tera>` 注入 handler；模板与静态资源由 `rust-embed` 在编译期嵌入，运行时不再读盘。
- **Session / Cookie**：使用 `tower_sessions::Session` 与 `tower_cookies`；签名密钥在 `main` 中用
  `Key::from(&rand::rng().random::<[u8; 64]>())` 生成。
- **关闭信号**：通过 `Extension(shutdown_tx)` 传入 handler，用于优雅关闭。

### 命名与风格

- **文件/函数/变量**：snake_case。**类型/枚举**：PascalCase。**常量**：SCREAMING_SNAKE_CASE。
- **调试输出**：使用 `#[cfg(debug_assertions)]` 包裹仅开发环境需要的 `print_info`/`print_error`，避免在生产日志中刷屏。

### 用户可见文案与注释

- 用户可见的提示、错误信息及代码注释使用**中文**，与现有代码风格一致。

---

## 文档与后续更新

- 依赖版本或 Rust/toolchain 变更时，应同步更新本文档中的“Technology Stack & Versions”，并再次核实时效性与官方文档。
- 新增错误类型、路由或业务常量时，建议在本文档的“Critical Implementation Rules”中补充对应规则，便于 AI 与人工保持一致。
