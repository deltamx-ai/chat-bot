# Chat Bot 修复提示词（基于 2026-05-15 代码审查）

你现在要修复一个 Rust + Tauri + React/TypeScript 项目中的一批已存在改动。请直接在当前代码基础上进行修复，不要重做架构，不要大范围重命名，也不要引入与本次问题无关的重构。

你的目标不是“优化代码风格”，而是：**消除真实问题、补上缺口、保证现有改动可安全合并。**

---

## 任务目标

请完成下面几件事：

1. 修复当前 review 中已经发现的真实问题。
2. 保持现有功能方向不变，尤其是：
   - Copilot Auth 流程
   - provider runtime / provider routing
   - 前端 pending message / pending ops 交互
3. 在修复后自行验证：
   - `cargo check` 通过
   - `cargo test` 通过（包括 doctest）
   - 前端 lint 通过
4. 如果发现 review 中提到的问题背后还有连带 bug，也一并修掉，但不要顺手做无关重构。

---

## 已确认的问题

### 1. `cargo test` 没有全绿，`core` 的 doctest 失败

现状：
- `cargo check` 已通过
- Rust 普通测试基本通过
- 但 `cargo test` 最终失败，失败点在：
  - `cargo test -p core --doc`
- 错误里出现了 `E0433`
- 这说明 `crates/core` 中存在 doctest 无法编译或解析的问题

你的任务：
- 定位 `crates/core` 里触发 doctest 的文档内容
- 修复导致 `E0433` 的文档示例或注释内容
- 如果某段示例本来就不是可执行代码，应明确改成：
  - `ignore`
  - `text`
  - 或改写成不会被 rustdoc 当成 doctest 的形式
- 不要粗暴删除有价值的文档说明，优先保留文档信息量

验收标准：
- `cargo test -p core --doc` 通过
- `cargo test` 全量通过

---

### 2. `ProviderKind::from_model_id()` 规则过于脆弱，存在 provider 误判风险

当前逻辑大致是按前缀判断：
- `claude*` => `Anthropic`
- `copilot*` => `Copilot`
- `gpt*` => `OpenAi`
- 其他 => `Custom`

这个实现有明显风险：
- OpenAI 新模型不一定都以 `gpt` 开头，比如：`o3`、`o4`、未来的新命名
- embedding / moderation / 其他模型命名也可能不符合当前规则
- 一旦误判成 `Custom`，后续 provider routing 可能走错实现

你的任务：
- 改进 `ProviderKind::from_model_id()` 或与之等价的 provider 推断逻辑
- 目标是：
  - 比当前前缀判断更稳健
  - 至少覆盖目前项目中实际会用到的 OpenAI / Anthropic / Copilot 场景
  - 不要为了“完美”做成超复杂的规则引擎
- 可以接受的修复方式包括但不限于：
  - 扩展已知前缀集合
  - 在模型选择结构里显式传入 provider kind，而不是只靠模型名猜
  - 对“猜测失败”的情况做更安全的 fallback
- 如果你选择调整数据结构或接口，请控制改动范围，不要把整条链路推翻重做

验收标准：
- 不再只依赖 `gpt` 前缀识别 OpenAI
- provider 选择逻辑与现有调用链保持一致
- 至少补上对应测试，覆盖几个代表性模型名

建议至少覆盖这些例子：
- `gpt-4o`
- `o3`
- `claude-3-7-sonnet`
- `copilot/claude-3.7-sonnet`
- 一个未知模型名

---

## 建议顺手检查的风险点

这些不一定已经是确定 bug，但在修复时建议一起确认：

### 3. provider 路由的 fallback 行为是否安全

当前看起来：
- `ProviderRegistry::provider_for(...)` 在 Copilot 不可用时，会 fallback 到 `AlmaProvider::default()`
- `OpenAi` / `Anthropic` / `Custom` 当前也可能统一走 `AlmaProvider::default()`

你需要确认：
- 这是不是符合设计预期
- 当 `ModelSelection.kind` 被误判时，是否会 silently 走到错误 provider
- 如果 fallback 是刻意设计，是否需要更清晰的注释、日志或测试来防止后续误用

如果这里有隐性问题，请以**最小改动**修复。

---

### 4. 前端 pending message 占位逻辑是否覆盖“新会话创建后真实 conversationId 回填”场景

当前前端改动里引入了：
- `pendingOpsStore`
- optimistic user / assistant placeholder message
- 发送消息时先写入 placeholder，再用真实返回值替换

你需要确认：
- 当 `conversation_id` 为空并走默认占位 `conv_default` 时，真实返回的 conversation id 回填是否完整
- placeholder 是否一定会被移除
- 会不会出现：
  - UI 里残留 pending assistant 空消息
  - 新旧 conversation key 下各留一份消息
  - 发送失败后 pending 状态未清理

如果已经没问题，可以只补测试或补注释；如果有缺口，就直接修。

---

### 5. auth 接口错误语义是否合理

当前 `/api/auth/copilot/*` 路由中：
- `AuthorizationPending / SlowDown / ExpiredToken` 被映射成 `409 Conflict`
- `Cancelled / AccessDenied / Forbidden` 被映射成 `403`

你需要确认：
- 这些状态码是不是前端当前消费逻辑所期望的
- 是否存在“其实应该返回 200 + 状态体，而不是 HTTP 错误”的场景
- poll 流程中前端是否可能因为状态码处理不一致而误判成 `unknown`

如果你确认现在设计没问题，可以保留；如果有明显不一致，就最小化修正。

---

## 工作约束

请严格遵守：

- 不要做无关重构
- 不要顺手改格式化风格之外的大量命名
- 不要删除现有功能
- 不要把一个小问题扩大成全面架构重写
- 优先补测试，而不是只口头“认为没问题”
- 所有修复必须和当前代码方向兼容

---

## 建议执行顺序

建议你按这个顺序做：

1. 先修 doctest，确保测试链路干净
2. 再修 provider kind / model routing 判断
3. 再检查 pending message 回填链路
4. 最后统一跑验证

---

## 最终输出要求

修复完成后，请输出一份简短结果总结，内容包括：

1. 你修了哪些文件
2. 每个问题的修复思路
3. 新增或修改了哪些测试
4. 最终验证结果：
   - `cargo check`
   - `cargo test`
   - 前端 lint
5. 如果你发现某个 review 项目最终判断“不是 bug”，请说明理由

输出风格务必简洁、工程化、可审阅，不要写成长篇空话。
