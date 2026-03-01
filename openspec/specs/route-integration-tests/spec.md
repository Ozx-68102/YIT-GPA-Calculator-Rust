# 关键路由集成测试规格

## ADDED Requirements

### Requirement: 无 session 访问 /result 时重定向到登录页

系统 SHALL 在用户未持有有效 session（或 session 中无课程数据）访问 GET `/result` 时，返回 302 重定向，且 `Location` 指向根路径
`/`（登录页），以便用户先登录或使用免登录方式获取数据。

#### Scenario: 无 cookie/session 时 GET /result 返回 302 且 Location 为 /

- **WHEN** 客户端发起 GET `/result` 且未携带 session cookie（或 session 中 `courses` 为空）
- **THEN** 响应状态码为 302，且响应头 `Location` 为 `/`（或等价于根路径的绝对 URL）

---

### Requirement: WebError/FileError 与 HTTP 状态码的映射可回归验证

系统 SHALL 将 `WebError` 与 `FileError` 的各变体映射为确定的 HTTP 状态码，且该映射 MUST 通过自动化测试（单元或集成）可回归验证，避免修改
handler 或 `IntoResponse` 时漏测。

#### Scenario: LoginFailed 映射为 401

- **WHEN** 请求导致 `WebError::WebScrapingError(WebScrapingError::LoginFailed)` 被返回
- **THEN** 响应状态码为 401 Unauthorized

#### Scenario: FileError 映射为 400

- **WHEN** 请求导致 `WebError::FileError`（如 `FileError::OpenError` 或 `NoValidDataFound`）被返回
- **THEN** 响应状态码为 400 Bad Request

#### Scenario: TemplateError 与 InternalError 映射为 500

- **WHEN** 请求导致 `WebError::TemplateError` 或 `WebError::InternalError` 被返回
- **THEN** 响应状态码为 500 Internal Server Error

#### Scenario: 其他 WebScrapingError 映射为 500

- **WHEN** 请求导致 `WebError::WebScrapingError` 且非 `LoginFailed`（如 `ParseError`、`InvalidCourseData` 等）被返回
- **THEN** 响应状态码为 500 Internal Server Error
