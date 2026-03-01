## Why

GPA 核心逻辑（分数→绩点转换、四舍五入、课程排除与加权平均）集中在 `src/business.rs`
，目前无单元测试。任何重构或修改都可能无意中改变等级制/百分制边界、排除规则或小数舍入行为，导致线上结果与预期不一致。为把「分数→绩点」和「课程列表→GPA」的边界用测试锁死，防止被改坏，需要为
`score_trans_grade`、`round_2decimal` 以及通过 `process_scraped_course_results` 暴露的 GPA/排除逻辑补充单元测试。

## What Changes

- 在 `src/business.rs` 中增加 `#[cfg(test)] mod tests`，对以下三个逻辑进行单元测试覆盖：
    - **score_trans_grade(score: &str) -> Option<Decimal>**：等级制（不及格/合格/中/良/优）、百分制边界点（0, 60, 64, 67, 70,
      74, 77, 80, 83, 87, 90, 95, 100）、非法输入（空串、非数字、多小数点、超范围等）→ `None`。
    - **round_2decimal(d: Decimal) -> Decimal**：保留 2 位小数、四舍五入、整数值不变等用例。
    - **GPA/排除逻辑**：通过 `process_scraped_course_results`（或必要时对 `calculate_gpa_from_list` 做 `pub(crate)`
      以直接测）覆盖：空列表→(0, [])；仅「入学教育」被 PERMANENT_IGNORED 排除；仅「公选」/「通识限选」在 Default 模式排除；仅含「体育」等
      EXCLUDED_COURSES_KEYWORD 在 Default 排除；正常若干门课加权平均与 round_2decimal 一致；Default 与 All 两种 mode 结果差异。
- 不新增路由、不修改 handler/scraping/models；不新增错误变体。仅影响 business 层测试代码与可能的可见性调整（如
  `calculate_gpa_from_list` 改为 `pub(crate)` 以便同 crate 内测试子模块访问，可选）。

## Capabilities

### New Capabilities

- `gpa-core-unit-tests`: 为 business 层 GPA 核心逻辑（score_trans_grade、round_2decimal、GPA 计算与课程排除）建立单元测试规范与用例，锁死边界行为。

### Modified Capabilities

- 无。仅新增测试与可选可见性调整，不改变现有接口或需求。

## Impact

- **影响范围**：仅 `src/business.rs`（新增 `#[cfg(test)] mod tests`，及可选地将 `calculate_gpa_from_list` 改为
  `pub(crate)`）。
- **其他层**：router、handler、scraping、models 不受影响；无新依赖（沿用现有 `rust_decimal` / `dec!` 与 `Course` 结构）。
- **注意事项**：测试数据需与 project-context
  中的排除常量（PERMANENT_IGNORED_COURSES、ATTR_EXCLUSIONS、EXCLUDED_COURSES_KEYWORD）及现有百分制/等级制规则一致，后续若调整常量或分段需同步更新测试用例。
