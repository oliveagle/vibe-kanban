# DDD 数据库重构 - 多 Agent 协作计划

## 任务拆分策略

### Agent A: Execution Process 模块 (DDD 重构)
**负责人**: Agent A
**范围**: 
- `crates/db/src/models/execution_process.rs`
- `crates/db/src/models/execution_process_repo_state.rs`

**任务清单**:
- [ ] 分析现有 SQL 查询，标记需要改为 JSONB 的查询
- [ ] 重写 `find_by_id` - 从 JSONB data 提取 executor_action, exit_code, dropped 等字段
- [ ] 重写 `find_by_session_id` - 使用 JSONB 查询
- [ ] 重写 `find_running` - 使用 JSONB 查询
- [ ] 重写 `create` - 插入时构建 JSONB data 对象
- [ ] 重写 `update` - 使用 jsonb_set 更新 JSONB 字段
- [ ] 运行 `cargo check --package db` 验证编译

**关键 SQL 模式**:
```rust
// 读取 JSONB 字段
(data->>'field_name')::target_type

// 写入 JSONB 字段
jsonb_set(data, '{field_name}', to_jsonb($1))

// COALESCE 处理 NULL
COALESCE((data->>'field')::type, default_value)
```

**验收标准**:
- [ ] 所有 SQL 查询使用 JSONB 操作符
- [ ] `cargo check --package db` 0 errors
- [ ] 单元测试通过 (如果存在)

---

### Agent B: Workspace & Session 模块 (DDD 重构)
**负责人**: Agent B
**范围**:
- `crates/db/src/models/workspace.rs`
- `crates/db/src/models/session.rs`
- `crates/db/src/models/workspace_repo.rs` (已部分完成，需审查)

**任务清单**:
- [ ] 审查 `workspace_repo.rs` 现有的 DDD 重构
- [ ] 重写 `workspace.rs` 中的查询:
  - `fetch_all` - 使用 JSONB 查询 workspaces
  - `fetch_all_bulk` - 批量查询使用 JSONB
  - `load_context` - 关联查询使用 JSONB
  - `resolve_container_ref` - 容器引用查询
- [ ] 重写 `session.rs` 中的查询:
  - `find_by_workspace_id` - 使用 JSONB 查询 sessions
  - `create` - 插入时构建 JSONB data
  - `update_status` - 使用 jsonb_set 更新
- [ ] 运行 `cargo check --package db` 验证编译

**关键 SQL 模式**:
```rust
// Workspace 查询示例
SELECT id, task_id, name, status, data, created_at, updated_at, deleted_at
FROM task_workspaces 
WHERE task_id = $1 AND deleted_at IS NULL

// Session 查询示例  
SELECT id, workspace_id, status, data, created_at, updated_at
FROM sessions
WHERE workspace_id = $1
```

**验收标准**:
- [ ] 所有 SQL 查询使用 DDD JSONB 模式
- [ ] `cargo check --package db` 0 errors
- [ ] 与 Agent A 的接口兼容

---

### Agent C: Task & Project 模块 (DDD 重构)
**负责人**: Agent C
**范围**:
- `crates/db/src/models/task.rs`
- `crates/db/src/models/project.rs`
- `crates/db/src/models/project_repo.rs` (已部分完成，需审查)

**任务清单**:
- [ ] 审查 `project_repo.rs` 现有的 DDD 重构
- [ ] 重写 `task.rs` 中的查询:
  - `find_by_project_id` - 使用 JSONB 查询 tasks
  - `find_with_workspace_count` - 聚合查询使用 JSONB
  - `create` - 插入时构建 JSONB data (包含 workspaces, sessions 等)
  - `update` - 使用 jsonb_set 更新嵌套字段
  - `update_status` - 更新 status 和 JSONB 中的相关字段
- [ ] 重写 `project.rs` 中的查询:
  - `find_all` / `find_by_user_id` - 使用 JSONB 查询 projects
  - `create` - 插入时构建 JSONB data (包含 repos, settings 等)
  - `update` - 使用 jsonb_set 更新嵌套字段
  - `find_with_repo_count` - 聚合查询使用 JSONB
- [ ] 运行 `cargo check --package db` 验证编译

**关键 SQL 模式**:
```rust
// Task 创建时构建 JSONB
jsonb_build_object(
  'title', $1,
  'description', $2,
  'workspaces', '[]'::jsonb,
  'assignee', $3,
  'priority', $4,
  'subtasks', '[]'::jsonb
)

// Project 更新时使用 jsonb_set
jsonb_set(
  data, 
  '{repos}', 
  data->'repos' || $1::jsonb
)
```

**验收标准**:
- [ ] 所有 SQL 查询使用 DDD JSONB 模式
- [ ] `cargo check --package db` 0 errors  
- [ ] 与 Agent A、B 的接口兼容

---

## 协作规范

### 数据库名称统一
**必须使用**: `vibe_kanban`
**禁止使用**: `vk_db` 或任何其他名称

环境变量设置:
```bash
export DATABASE_URL="postgres://vibekanban:vibekanban123@localhost:5632/vibe_kanban"
```

### 代码审查清单
- [ ] SQL 查询使用 JSONB 操作符 (`->`, `->>`, `@>`, `jsonb_set`)
- [ ] 使用 `COALESCE` 处理 NULL 值
- [ ] 软删除使用 `deleted_at IS NULL`
- [ ] 创建 partial index 排除已删除数据
- [ ] 复杂 JSONB 查询创建 GIN index

### 冲突解决
如果发现多个 agent 修改了同一文件:
1. 先合并到 dev 分支
2. 解决冲突时优先保留 JSONB 模式代码
3. 删除传统关系型查询代码
4. 运行 `cargo check` 验证

### 测试策略
每个模块重构完成后:
1. 运行 `cargo check --package db` - 必须 0 errors
2. 运行 `cargo test --package db` - 所有测试通过
3. 手动测试后端启动 `just dev-srv` - 容器正常启动

---

## 时间表

| 阶段 | 模块 | 负责人 | 预计时间 | 依赖 |
|------|------|--------|----------|------|
| 1 | Execution Process | Agent A | 2-3天 | 无 |
| 2 | Workspace & Session | Agent B | 2-3天 | 无 |
| 3 | Task & Project | Agent C | 2-3天 | 无 |
| 4 | 集成测试 | 所有人 | 1-2天 | 1,2,3完成 |
| 5 | 文档更新 | 所有人 | 0.5天 | 4完成 |

**总预计时间**: 7-10 天

---

## 紧急联系

如有问题或需要协调，请通过以下方式联系：
- Slack: #vibe-kanban-dev
- Email: dev@vibe-kanban.com
- 每日站会: 10:00 AM (UTC+8)

---

**准备开始！请选择你的任务并更新状态。**