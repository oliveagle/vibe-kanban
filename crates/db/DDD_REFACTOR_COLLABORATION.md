# DDD 数据库重构 - 协作文档

## 当前状态 (2025-02-08)

### 问题背景
当前数据库仍然使用传统的关联表结构（`repos`, `workspace_repos`, `project_repos` 等 30+ 张表），但 AGENTS.md 和 commit `35fbb8b` 显示 DDD 重构计划已经存在。

### 关键发现
1. **commit `35fbb8b`** 声称完成了 DDD 重构，但实际 SQL 迁移文件仍然是传统表结构
2. **workspace_repo.rs 和 project_repo.rs** 仍然查询已删除的表（在 DDD 架构中这些表应该被 JSONB 替代）
3. **134 处代码引用** 这些即将被重构的模块

### 已完成的准备工作
1. ✅ 创建了新的 DDD 迁移文件: `20250618000000_ddd_aggregate_roots.sql`
   - 定义了 5 个聚合根表: `users`, `projects`, `tasks`, `task_workspaces`, `execution_processes`, `events`
   - 每个表都有 `data JSONB` 字段存储领域数据
   - 包含软删除 (`deleted_at`) 和视图

### 当前任务分配

#### Agent A (当前 Agent): 重构 workspace_repo.rs
**目标**: 将 `workspace_repo.rs` 从查询 `workspace_repos` 表改为操作 `task_workspaces.data` JSONB

**数据结构**:
```rust
// WorkspaceRepo 结构体现在存储在 workspace.data["repos"] 中
pub struct WorkspaceRepo {
    pub id: Uuid,              // repo_id
    pub name: String,          // repo name
    pub path: Option<String>, // repo path
    pub target_branch: String,
    pub created_at: DateTime<Utc>,
}
```

**需要修改的函数**:
1. `create_many` - 插入 repos 到 workspace.data
2. `find_by_workspace_id` - 从 workspace.data 读取
3. `find_repos_with_target_branch_for_workspace` - 查询 workspace.data
4. `update_target_branch` - 修改 workspace.data
5. `find_unique_repos_for_task` - 跨 workspaces 查询

**SQL 模式示例**:
```rust
// 读取 repos
let repos: Vec<WorkspaceRepo> = sqlx::query_scalar::<_, serde_json::Value>(
    "SELECT data->'repos' FROM task_workspaces WHERE id = $1"
)
.bind(workspace_id)
.fetch_optional(pool)
.await?
.map(|json| serde_json::from_value(json).unwrap_or_default())
.unwrap_or_default();

// 写入 repos
let repos_json = serde_json::to_value(&repos)?;
sqlx::query(
    "UPDATE task_workspaces 
     SET data = jsonb_set(COALESCE(data, '{}'::jsonb), '{repos}', $1::jsonb)
     WHERE id = $2"
)
.bind(&repos_json)
.bind(workspace_id)
.execute(pool)
.await?;
```

#### Agent B (协作 Agent): 重构 project_repo.rs
**目标**: 将 `project_repo.rs` 从查询 `project_repos` 表改为操作 `projects.data` JSONB

**数据结构**:
```rust
// ProjectRepo 结构体现在存储在 project.data["repos"] 中
pub struct ProjectRepo {
    pub id: Uuid,              // project_repo id
    pub project_id: Uuid,
    pub repo_id: Uuid,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
}
```

**需要修改的函数**:
1. `find_by_project_id` - 从 project.data 读取 repos
2. `find_by_repo_id` - 查询所有包含该 repo 的 projects
3. `find_by_project_id_with_names` - 带 repo 名称查询
4. `find_repos_for_project` - 获取 project's repos
5. `add_repo_to_project` - 添加 repo 到 project.data
6. `remove_repo_from_project` - 从 project.data 移除 repo

### 依赖关系与协作要点

1. **Repo 结构体**: 两个 Agent 都需要使用 `repo::Repo` 结构体，需要保持一致
2. **类型导出**: `RepoWithTargetBranch`, `RepoWithCopyFiles` 等类型需要在 TS 中可用
3. **测试**: `task_server_tests.rs` 使用了这些类型，需要同步更新

### 下一步行动

1. **Agent A** 开始重构 `workspace_repo.rs`:
   - 重新定义 `WorkspaceRepo` 结构体（存储在 JSONB 中）
   - 重写所有 SQL 查询为 JSONB 操作
   - 保持对外接口兼容

2. **Agent B** 开始重构 `project_repo.rs`:
   - 重新定义 `ProjectRepo` 结构体（存储在 JSONB 中）
   - 重写所有 SQL 查询为 JSONB 操作
   - 保持对外接口兼容

3. 完成后一起运行 `cargo check` 和测试

### 关键代码模式

```rust
// 从 JSONB 读取 Vec<T>
let items: Vec<T> = row.data
    .get("key")
    .and_then(|v| serde_json::from_value(v.clone()).ok())
    .unwrap_or_default();

// 写入 Vec<T> 到 JSONB
let json = serde_json::json!({ "key": items });
sqlx::query("UPDATE table SET data = $1 WHERE id = $2")
    .bind(json)
    .bind(id)
    .execute(pool)
    .await?;
```

---

**最后更新**: 2025-02-08
**协作者**: Agent A (workspace_repo.rs), Agent B (project_repo.rs)
