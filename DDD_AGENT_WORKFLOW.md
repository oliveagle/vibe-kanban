# DDD 重构协作指南 - 多 Agent 并行工作

## 🎯 目标
使用 Git Worktree 模式，让多个 Agent 并行开发，避免代码冲突。

---

## 🌳 Git Worktree 模式

### 什么是 Worktree？
git worktree 允许你在同一个仓库中同时拥有多个工作目录，每个目录可以独立开发不同的分支。

### 为什么用 Worktree？
g| 优势 | 说明 |
|-------|-------|
| **避免冲突** | 每个 Agent 在自己的 worktree 工作，互不干扰 |
| **快速切换** | 无需 `git stash` 或 `git checkout`，直接进目录 |
| **独立环境** | 每个 worktree 可以有不同的未提交修改 |
| **并行开发** | 多个 Agent 同时工作在同一个仓库的不同部分 |

---
n## 📁 目录结构

```
/mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/          # 主仓库 (main worktree)
├── crates/
│   └── db/
│       └── src/
│           └── models/
│               ├── execution_process.rs          # Agent A 修改的文件
│               ├── workspace.rs               # Agent B 修改的文件
│               ├── task.rs                  # Agent C 修改的文件
│               └── ...
├── agent-a-worktree/        # Agent A 的 worktree (git worktree add 创建)
│   └── crates/db/src/models/execution_process.rs   # 并行修改
├── agent-b-worktree/        # Agent B 的 worktree
│   └── crates/db/src/models/workspace.rs
└── agent-c-worktree/        # Agent C 的 worktree
    └── crates/db/src/models/task.rs
```

---

## 🔧 初始化步骤

### Step 1: 创建 Worktree (每个 Agent 执行一次)
```bash
# Agent A 创建自己的 worktree
git worktree add ../agent-a-worktree -b feature/execution-process-ddd


# Agent B 创建自己的 worktree  
git worktree add ../agent-b-worktree -b feature/workspace-session-ddd


# Agent C 创建自己的 worktree
git worktree add ../agent-c-worktree -b feature/task-project-ddd
```

### Step 2: 在 Worktree 中开发
```bash
# 进入自己的 worktree
cd ../agent-a-worktree

# 开始修改文件
vim crates/db/src/models/execution_process.rs
```
git worktree list  # 查看所有 worktree 状态
```
git worktree prune    # 清理已删除的 worktree
```
git worktree remove ../agent-a-worktree  # 删除 worktree
```
g### Step 3: 同步到主仓库n在各自 worktree 完成修改后：
g```bash
# 1. 在 worktree 中提交
git add .
git commit -m "Agent A: DDD refactor execution_process"

# 2. 推送到远程分支 (可选)ngit push origin feature/execution-process-ddd
# 3. 回到主仓库合并
cd /mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban
git merge feature/execution-process-ddd
```
g### Step 4: 清理 Worktree (完成后)
```bash
git worktree remove ../agent-a-worktree --force
```
g---
g## 📝 提交规范
### Commit Message 格式
```
git commit -m "[Agent A] refactor: Execution Process DDD JSONB"
```
g必须包含：
g- `[Agent X]` 前缀标识是哪个 Agent
- 类型：`refactor`, `feat`, `fix`, `test`
- 简短描述
- 英文描述 (遵循项目规范)
n### 推送前检查
```bash
# 1. 检查编译
cargo check --package db
# 2. 格式化
cargo fmt
# 3. 运行测试 (如果有)
cargo test --package db
```
g---
g## 🚨 冲突处理
如果发生冲突：
g1. 暂停工作，通知协调人
2. 使用 `git mergetool` 解决
3. 重新运行测试验证
4. 强制推送前再次确认

---
g## 📊 进度报告
每个 Agent 每30分钟在 Slack 报告：
- 当前任务状态
- 遇到的 blockers
- 预计完成时间
---

**协作文档**: `/mnt/volume3/data/repos/github.com/oliveagle/vibe-kanban/DDD_MULTI_AGENT_PLAN.md`