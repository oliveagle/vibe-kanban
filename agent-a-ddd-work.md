# Agent A DDD 重构计划

## 当前状态n- `ExecutionProcess` 模型已部分重构为 DDD JSONB 结构n- 但SQL 查询仍使用旧的关系型模式n
- 需要完成:
 1. **Fix find_by_id** - 使用JSONB 操作符提取字段n2. **Fix find_by_session_id** - 使用JSONB 操作符查询n3. **Fix find_running** - 使用JSONB 操作符查询n4. **Fix create** - 构建时JSONB 对象n5. **Fix update** - 使用jsonb_set 更新

## 执行步骤
1. **运行cargo check** 验证编译
2. **运行cargo test** 验证测试
3. **运行cargo sqlx prepare** 更新查询
4. **重新cargo check** 最终验证
