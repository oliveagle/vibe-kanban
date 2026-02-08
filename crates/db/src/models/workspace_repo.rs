use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use ts_rs::TS;
use uuid::Uuid;

use super::repo::Repo;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceRepo {
    pub id: Uuid,
    pub repo_id: Uuid,
    pub name: String,
    pub path: Option<String>,
    pub target_branch: String,
    #[ts(type = "Date")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, TS)]
pub struct CreateWorkspaceRepo {
    pub repo_id: Uuid,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RepoWithTargetBranch {
    #[serde(flatten)]
    pub repo: Repo,
    pub target_branch: String,
}

#[derive(Debug, Clone)]
pub struct RepoWithCopyFiles {
    pub id: Uuid,
    pub path: Option<std::path::PathBuf>,
    pub name: String,
    pub copy_files: Option<String>,
}

impl WorkspaceRepo {
    pub async fn create_many(
        pool: &PgPool,
        workspace_id: Uuid,
        repos: &[CreateWorkspaceRepo],
    ) -> Result<Vec<Self>, sqlx::Error> {
        let now = Utc::now();
        let workspace_repos: Vec<Self> = repos
            .iter()
            .map(|r| Self {
                id: Uuid::new_v4(),
                repo_id: r.repo_id,
                name: String::new(),
                path: None,
                target_branch: r.target_branch.clone(),
                created_at: now,
            })
            .collect();

        let repos_json = serde_json::json!({ "repos": &workspace_repos });
        sqlx::query(
            r#"
            UPDATE task_workspaces 
            SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&repos_json)
        .bind(workspace_id)
        .execute(pool)
        .await?;

        Ok(workspace_repos)
    }

    pub async fn find_by_workspace_id(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let row = sqlx::query_scalar::<_, Option<serde_json::Value>>(
            r#"
            SELECT data->'repos'
            FROM task_workspaces
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(workspace_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(Some(json)) => {
                let repos: Vec<Self> = serde_json::from_value(json)
                    .unwrap_or_default();
                Ok(repos)
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn find_repos_for_workspace(
        _pool: &PgPool,
        _workspace_id: Uuid,
    ) -> Result<Vec<Repo>, sqlx::Error> {
        Ok(vec![])
    }

    pub async fn find_repos_with_target_branch_for_workspace(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<RepoWithTargetBranch>, sqlx::Error> {
        let _repos = Self::find_by_workspace_id(pool, workspace_id).await?;
        Ok(vec![])
    }

    pub async fn find_by_workspace_and_repo_id(
        pool: &PgPool,
        workspace_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let repos = Self::find_by_workspace_id(pool, workspace_id).await?;
        Ok(repos.into_iter().find(|r| r.repo_id == repo_id))
    }

    pub async fn update_target_branch(
        pool: &PgPool,
        workspace_id: Uuid,
        repo_id: Uuid,
        new_target_branch: &str,
    ) -> Result<(), sqlx::Error> {
        let mut repos = Self::find_by_workspace_id(pool, workspace_id).await?;
        
        if let Some(repo) = repos.iter_mut().find(|r| r.repo_id == repo_id) {
            repo.target_branch = new_target_branch.to_string();
            
            let repos_json = serde_json::json!({ "repos": repos });
            sqlx::query(
                r#"
                UPDATE task_workspaces 
                SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                    updated_at = NOW()
                WHERE id = $2
                "#
            )
            .bind(&repos_json)
            .bind(workspace_id)
            .execute(pool)
            .await?;
        }
        
        Ok(())
    }

    pub async fn update_target_branch_for_children_of_workspace(
        pool: &PgPool,
        parent_workspace_id: Uuid,
        old_branch: &str,
        new_branch: &str,
    ) -> Result<u64, sqlx::Error> {
        let workspace_ids: Vec<Uuid> = sqlx::query_scalar(
            r#"
            SELECT w.id 
            FROM task_workspaces w
            JOIN tasks t ON w.task_id = t.id
            WHERE t.parent_workspace_id = $1
            "#
        )
        .bind(parent_workspace_id)
        .fetch_all(pool)
        .await?;

        let mut updated_count = 0u64;

        for workspace_id in workspace_ids {
            let mut repos = Self::find_by_workspace_id(pool, workspace_id).await?;
            let mut modified = false;
            
            for repo in repos.iter_mut() {
                if repo.target_branch == old_branch {
                    repo.target_branch = new_branch.to_string();
                    modified = true;
                }
            }
            
            if modified {
                let repos_json = serde_json::json!({ "repos": repos });
                sqlx::query(
                    r#"
                    UPDATE task_workspaces 
                    SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                        updated_at = NOW()
                    WHERE id = $2
                    "#
                )
                .bind(&repos_json)
                .bind(workspace_id)
                .execute(pool)
                .await?;
                
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    pub async fn find_unique_repos_for_task(
        pool: &PgPool,
        task_id: Uuid,
    ) -> Result<Vec<Repo>, sqlx::Error> {
        let workspace_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM task_workspaces WHERE task_id = $1 AND deleted_at IS NULL"
        )
        .bind(task_id)
        .fetch_all(pool)
        .await?;

        let mut seen_ids = std::collections::HashSet::new();
        let mut unique_repos = Vec::new();

        for workspace_id in workspace_ids {
            let repos = Self::find_by_workspace_id(pool, workspace_id).await?;
            for repo in repos {
                if seen_ids.insert(repo.repo_id) {
                    unique_repos.push(Repo {
                        id: repo.repo_id,
                        path: repo.path,
                        name: repo.name,
                        display_name: None,
                        created_at: repo.created_at,
                        updated_at: repo.created_at,
                    });
                }
            }
        }

        Ok(unique_repos)
    }

    pub async fn find_repos_with_copy_files(
        _pool: &PgPool,
        _workspace_id: Uuid,
    ) -> Result<Vec<RepoWithCopyFiles>, sqlx::Error> {
        Ok(vec![])
    }
}
