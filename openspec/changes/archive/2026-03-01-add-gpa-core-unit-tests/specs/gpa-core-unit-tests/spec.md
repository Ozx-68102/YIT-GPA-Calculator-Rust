# Spec: gpa-core-unit-tests

为 business 层 GPA 核心逻辑建立单元测试规范，锁死分数→绩点、四舍五入与课程排除边界行为。

## ADDED Requirements

### Requirement: score_trans_grade 单元测试覆盖

对 `score_trans_grade(score: &str) -> Option<Decimal>` 的单元测试 SHALL 覆盖以下行为，且与当前 `src/business.rs` 实现一致。

#### Scenario: 等级制映射正确

- **WHEN** `score` 为「不及格」或「不合格」
- **THEN** 返回 `Some(0)`

#### Scenario: 等级制及格与合格

- **WHEN** `score` 为「及格」或「合格」
- **THEN** 返回 `Some(1)`

#### Scenario: 等级制中良优

- **WHEN** `score` 分别为「中」「良」「优」
- **THEN** 依次返回 `Some(2.33)`、`Some(3.33)`、`Some(4.33)`（Decimal，保留两位）

#### Scenario: 百分制边界点映射

- **WHEN** `score` 为百分制字符串 0、60、64、67、70、74、77、80、83、87、90、95、100（及范围内代表值）
- **THEN** 返回与当前实现一致的绩点：0 与 59→0；60–63→1.33；64–66→1.67；…；95–100→4.67

#### Scenario: 百分制边界精确值

- **WHEN** `score` 为 "0"、"60"、"64"、"67"、"70"、"74"、"77"、"80"、"83"、"87"、"90"、"95"、"100"
- **THEN** 分别得到当前实现规定的绩点（0、1.33、1.67、2.00、2.33、2.67、3.00、3.33、3.67、4.00、4.33、4.67 等），无遗漏

#### Scenario: 非法输入返回 None

- **WHEN** `score` 为空串 ""、非数字 "abc"、多小数点 "60.5.5"、超范围 "101" 或 "-1"
- **THEN** 返回 `None`

---

### Requirement: round_2decimal 单元测试覆盖

对 `round_2decimal(d: Decimal) -> Decimal` 的单元测试 SHALL 验证保留 2 位小数与四舍五入行为。

#### Scenario: 保留两位小数

- **WHEN** 输入为多位小数的 Decimal
- **THEN** 输出为保留 2 位小数（与 `round_dp(2)` 一致）

#### Scenario: 四舍五入

- **WHEN** 输入的小数部分需要四舍五入（如 2.335、2.334）
- **THEN** 输出为四舍五入后的 2 位小数

#### Scenario: 整数值不变

- **WHEN** 输入为整数值（如 3、4.00）
- **THEN** 输出数值不变且仍为 2 位小数形式

---

### Requirement: GPA 与课程排除逻辑单元测试覆盖

通过 `process_scraped_course_results`（或可选地直接调用 `calculate_gpa_from_list`）的单元测试 SHALL 覆盖以下边界，且与
business 内 PERMANENT_IGNORED_COURSES、ATTR_EXCLUSIONS、EXCLUDED_COURSES_KEYWORD 一致。

#### Scenario: 空列表

- **WHEN** 传入空课程列表
- **THEN** 得到的 default（若存在）与 all 的 GPA 为 0，courses 为空

#### Scenario: 仅入学教育被永久排除

- **WHEN** 课程列表仅含「入学教育」一门课
- **THEN** 两套结果中该课均不参与计算，GPA 为 0，courses 为空（或等效）

#### Scenario: 仅公选或通识限选在 Default 排除

- **WHEN** 课程列表仅含属性为「公选」或「通识限选」的课程
- **THEN** Default 模式下这些课被排除，All 模式下计入；两种 mode 结果有差异

#### Scenario: 仅含体育等关键词在 Default 排除

- **WHEN** 课程列表仅含名称包含 EXCLUDED_COURSES_KEYWORD 的课（如「体育」）
- **THEN** Default 模式下被排除，All 模式下计入；两种 mode 结果有差异

#### Scenario: 正常若干门课加权平均一致

- **WHEN** 传入多门有效课程（含学分与绩点）
- **THEN** 计算得到的 GPA 与「总 credit_gpa / 总 credit」经 round_2decimal 后的值一致

#### Scenario: Default 与 All 结果差异

- **WHEN** 传入同时包含「计入 Default」与「仅计入 All」的课程
- **THEN** default.gpa 与 all.gpa 可不同，default.courses 与 all.courses 数量/内容不同，且符合当前排除规则
