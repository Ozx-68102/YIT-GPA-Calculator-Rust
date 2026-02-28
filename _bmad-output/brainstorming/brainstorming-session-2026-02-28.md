---
stepsCompleted: [1, 2, 3]
inputDocuments: []
session_topic: '围绕 YIT-GPA-Calculator-Rust：在现有架构与需求已满足的基础上，探索 BMAD 视角下会考虑而用户尚未想到的点；顺带可涉及 Rust 与 Python/PHP/HTML 的对比观察'
session_goals: '获得 BMAD 带来的惊喜与补充思路；识别差距与可改进方向；可选地沉淀 Rust vs 其他语言的观察'
selected_approach: 'ai-recommended'
techniques_used: ['Assumption Reversal', 'Alien Anthropologist', 'Question Storming']
ideas_generated: []
context_file: '_bmad-output/project-context.md'
---

# Brainstorming Session Results

**Facilitator:** David
**Date:** 2026-02-28

## Session Overview

**Topic:** 围绕 YIT-GPA-Calculator-Rust：在现有架构与需求已满足的基础上，探索「BMAD 视角下会考虑、而你尚未想到」的点；顺带可涉及
Rust 与 Python/PHP/HTML 的对比观察。

**Goals:** 获得 BMAD 带来的「惊喜」与补充思路；识别差距与可改进方向；可选地沉淀一些 Rust vs 其他语言的观察。

### Context Guidance

项目上下文已从 `project-context.md` 加载：技术栈（Rust 2024、axum、tera 等）、分层与错误处理规则、GPA/学分用
Decimal、排除规则等均已明确。头脑风暴可在此约束下探索功能、体验、架构、可维护性、边界情况或跨语言对比等方向。

### Session Setup

用户已确认会话参数。下一步：选择技巧运用方式。

## Technique Selection

**Approach:** AI-Recommended Techniques  
**Analysis Context:** 围绕 YIT-GPA-Calculator-Rust，目标为发现 BMAD 会考虑而用户尚未想到的点，并可选地沉淀 Rust vs
其他语言的观察。

**Recommended Techniques:**

- **假设反转（Assumption Reversal）：** 系统地质疑「项目已经够用」背后的隐含假设，找出若反转会暴露的盲区与新可能；对应「BMAD
  会考虑而我没想到」的发现。
- **外星人类学家（Alien Anthropologist）：** 用完全外来视角（如只懂 Python/PHP/前端的人、或 BMAD 评审）审视项目，产生「外人会问的问题」与
  Rust/架构对比素材。
- **问题风暴（Question Storming）：** 先只提问题不答，把前两阶段的发现收束成可跟进的问题清单与 3–5 条可行动项。

**AI Rationale:** 目标为发现盲区与换视角，故选用「深度质疑假设」+「外来视角」+「问题收束」的三段式；兼顾反思型会话风格与个人项目、无商业压力等约束。

---

## Technique Execution Results

### 1. 假设反转（Assumption Reversal）— 已完成

**本阶段做了什么：** 用户先列出项目中的隐含假设（成绩来源与稳定性、核心=算绩点、脚本形态/无 RBAC、技术选型
Rust、本地运行、无持久化、单校/小范围使用等），再对 A/B/G 三条「反转可能性」按实际情况回应，不做进一步反转。

**主要产出：**

- **假设清单（整理）：** 成绩来自教务系统且结构稳定；可不登录使用绩点计算；核心是算绩点、查分为前置；项目为完整脚本、无多用户/权限；Rust
  选型满足需求；本地跑、不部署；数据不持久化、用 Session；仅本人与少数同学、本校、无需无障碍/国际化。
- **用户对「反转」的回应：**
    - **A（教务改版/不能爬）：** 四年观察教务系统稳定；若有问题有免登录上传文件兜底；Excel 内已做限制与约束，不赘述。
    - **B（扩展为查分/通知/历史）：** 当前需求就是算绩点，查分用户可上教务系统；数据依赖教务
      response，查不到是教务侧问题；教务本身有成绩历史，扩展必要性低。
    - **G（传播/多校/开源）：** 已放学校代码库与 GitHub；Python 版已被他人做成服务器版并传播，本 Rust 版影响力有限，顺其自然；README
      已写清，提供打包 exe、几 MB、双击即用。
- **收尾结论：** 用户认为信息已足够详细，无需再对更多假设做反转；从引导方视角也认可该阶段已达成「把假设显性化 + 有理由的边界」目标。

**创意/协作要点：** 用户主动用「按实际情况回答、不做反转」的方式参与，同样构成有效的假设审视；补充了兜底设计、Excel
约束、传播与文档等细节，使项目边界更清晰。

---

### 2. 外星人类学家（Alien Anthropologist）— 已完成

**本阶段做了什么：** 用户表示难以代入「外人」身份（项目是随心、内部驱动），故改为由引导方扮演「外人」发问（身份 A：只懂
Python/PHP/前端的人；身份 B：BMAD 式评审），用户逐条回应。

**外人问题与用户回应摘要：**

| 问题                                  | 身份 | 用户回应要点                                                                                                                           |
|-------------------------------------|----|----------------------------------------------------------------------------------------------------------------------------------|
| 为啥用 Rust？Python 版不够吗？               | A  | 拿到 Python 源码时教务刚更新，旧接口被删、脚本不可用；维护后想起 Rust 帖（严格、编译检查、比 C 安全、风格像 Python）；不喜欢 Python 的「运行时才暴露问题」，偏好确定性；Rust 借用检查把问题掐在编译前 + 性能好，觉得值。 |
| Result / match 和 PHP/Python 异常比不累吗？ | A  | 同上，追求编译期确定性，不图少写几行。                                                                                                              |
| 单一 exe、无需运行时是刻意选 Rust 的理由吗？         | A  | 不是。选了 Rust 之后才发现可以这样，属于「意外之喜」。                                                                                                   |
| 教务改版后有无降级路径？（如只靠上传 Excel）           | B  | 无效问题——已有免登录上传文件功能。                                                                                                               |
| 别的学校想用要改什么？有没有「如何适配新学校」文档？          | B  | 无效问题——明确只适配本校；新学校应另起项目。                                                                                                          |
| 出错时用户看到什么？是否友好？                     | B  | 有 Bootstrap 5，异常时有详细弹窗。exe 会唤起 Windows 终端，关终端即关程序，这点或许可优化；当前以 Web 服务器形式存在，终端是必须的。懒得做原生 UI，且自认美术一般。                               |

**Rust vs 其他语言的观察（沉淀）：** 选型动机 = 教务接口失效后重写 + 偏好「编译期确定性」胜过「少写代码」；单 exe 为选型后的意外收获。
**可跟进点：** 关终端即退出的体验是否要优化（如最小化到托盘、或说明「勿关终端」）；原生 UI 暂不做、可留作后续可选方向。

---

### 3. 问题风暴（Question Storming）— 基于通读项目后的修订版

**说明：** 引导方在用户要求下通读了 `README.md` 与项目代码（`src/`、`templates/`
、路由与错误类型等），据此重新整理「值得保留的问题」清单，使问题与真实实现和文档一一对应。

**代码与文档要点（供问题锚定）：**

- **入口与运行：** `main.rs` 绑定 127.0.0.1:8080、webbrowser 自动打开、graceful shutdown via `Extension(shutdown_tx)`
  ；README 已写明「请勿关闭终端窗口」。
- **路由与职责：** `router.rs` 定义 `/`（登录）、`/score-from-official-website`、`/score-from-file`、`/download-template`、
  `/result`、`/recalc`、`/logout`、`/shutdown`；`handler` 层负责 Session 读写（如 `gpa_default` / `courses_default` /
  `result_mode`）、错误通过 `WebError::IntoResponse` 映射到 HTTP 状态与文案。
- **错误与用户可见信息：** `models.rs` 中 `WebScrapingError`（如 `LoginFailed`、`TeachingEvaluatingNotAccomplish`、
  `HostDeprecated` 等）、`FileError`（如 `NoValidDataFound`）；前端通过 Bootstrap Toast/Modal 展示后端返回的 message。
- **业务与数据：** `business.rs` 中 Default/All 两种 GPA 模式、`ResultSource::OfficialWebsite` vs `InputFile`；免登录模式仅写入
  `gpa_all`/`courses_all`，无 default。Excel 解析依赖 `Sheet1`、跳过前 3 行、列顺序为名称/学分/分数。
- **爬虫与配置：** `scraping.rs` 使用 `AAOWebsite`、`_url.rs` 中 `HOST = "jw.yit.edu.cn"`；README 注意事项含「代理可能导致
  HTTP 请求失败」「绩点仅供参考」等。

---

**问题清单（基于实际代码与 README，只列问题、不要求当下有答案）：**

1. **终端与进程生命周期**  
   README 与 `main` 已明确「请勿关闭终端窗口」。若用户误关终端，当前行为是进程直接退出。是否需要在 README
   或首次打开页面上更醒目地重复该提示（或考虑「最小化到托盘」等可选优化）？

2. **错误文案与前端展示**  
   `WebScrapingError` / `FileError` 的 `#[error(...)]` 文案会经 `IntoResponse` 变成 HTTP body，前端用 Toast
   展示。是否希望区分「给用户看的简短说明」与「仅 debug 的详细信息」（例如生产环境不返回解析堆栈）？

3. **免登录 Excel 模板的契约**  
   当前实现假定 Excel 为 `Sheet1`、前 3 行表头、列顺序固定。若将来模板格式变更（如多 Sheet、列顺序调整），是否打算在 README
   或下载页注明「请勿修改模板结构」，或在代码中增加更明确的格式校验与错误提示？

4. **教务域名或路径变更**  
   `_url.rs` 中 `HOST` 与爬虫路径硬编码。若学校更换域名或路径（如 `jw.yit.edu.cn` → 新域名），是否考虑将「 base URL
   或关键路径」做成配置（如环境变量/配置文件），便于以后只改一处？

5. **Rust 选型叙事与文档**  
   当前 README 侧重功能与使用。是否愿意在「项目简介」或「开发与编译」中加一小段「为何用 Rust」（编译期确定性、单 exe 无运行时、与
   Python 版的承接关系），方便后续自己或他人理解项目背景？

6. **代理与网络失败的可见性**  
   README 已写「使用代理时可能 HTTP 请求失败，请临时关闭代理」。用户遇到该情况时，后端返回的通常是
   `WebScrapingError::HttpRequest(...)`。是否需要在错误提示中显式出现「若使用代理可尝试关闭」等引导语（或在 README 中再次强调该条）？

7. **关闭程序流程与前端状态**  
   点击「关闭程序」会 POST `/shutdown`，服务端 graceful
   shutdown；前端有「程序已成功关闭，请手动关闭此浏览器选项卡」等文案。是否希望补充「关闭后请勿再点击其他按钮」或「终端窗口将自动关闭」之类说明，避免用户困惑？

---

**可跟进项（收束为 3～5 条，供后续选做）：**

| # | 可跟进项                                                       | 依据              |
|---|------------------------------------------------------------|-----------------|
| 1 | 终端/退出体验：在 README 或首屏再强调「请勿关闭终端」；或评估最小化到托盘等方案               | 问题 1、会话中外星人视角讨论 |
| 2 | 错误展示：区分用户向文案与调试信息；必要时在生产环境限制返回内容                           | 问题 2            |
| 3 | 模板契约：在 README 或下载处注明 Excel 格式要求；可选在代码中加强校验与报错              | 问题 3            |
| 4 | 教务 base URL/路径：评估是否配置化（如 env 或 config），便于日后域名/路径变更         | 问题 4            |
| 5 | 文档：在 README 中增加简短「为何用 Rust」；可选在错误提示或 README 中强化「代理导致失败」的说明 | 问题 5、6、7        |

以上问题与可跟进项均已写入本会话文档；是否采纳或调整可由你在后续开发中自行决定。
