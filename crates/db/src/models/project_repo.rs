use std::path::Path;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

use super::repo::Repo;

#[derive(Debug, Error)]
pub enum ProjectRepoError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Repository not found")]
    NotFound,
    #[error("Repository already exists in this project")]
    AlreadyExists,
}

/// ProjectRepo stored in project.data["repos"] JSONB
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ProjectRepo {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
}

/// ProjectRepo with repo details for script execution
#[derive(Debug, Clone)]
pub struct ProjectRepoWithName {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repo_id: Uuid,
    pub repo_name: String,
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: bool,
}

#[derive(Debug, Clone, Deserialize, TS)]
pub struct CreateProjectRepo {
    pub display_name: String,
    pub git_repo_path: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct UpdateProjectRepo {
    pub setup_script: Option<String>,
    pub cleanup_script: Option<String>,
    pub copy_files: Option<String>,
    pub parallel_setup_script: Option<bool>,
}

impl ProjectRepo {
    /// Find repos by project id from project.data JSONB
    pub async fn find_by_project_id(
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let row = sqlx::query_scalar::<_, Option<serde_json::Value>>(
            r#"
            SELECT data->'repos'
            FROM projects
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(project_id)
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

    /// Find projects containing a specific repo
    pub async fn find_by_repo_id(
        pool: &PgPool,
        repo_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let rows = sqlx::query_scalar::<_, Option<serde_json::Value>>(
            r#"
            SELECT data->'repos'
            FROM projects
            WHERE deleted_at IS NULL
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut matching_repos = Vec::new();
        for row in rows.into_iter().flatten() {
            if let Ok(repos) = serde_json::from_value::<Vec<Self>>(row) {
                for repo in repos {
                    if repo.repo_id == repo_id {
                        matching_repos.push(repo);
                    }
                }
            }
        }

        Ok(matching_repos)
    }

    /// Find repos with names for a project
    pub async fn find_by_project_id_with_names(
        pool: &PgPool,
        project_id: Uuid,
    ) -> Result<Vec<ProjectRepoWithName>, sqlx::Error> {
        let repos = Self::find_by_project_id(pool, project_id).await?;
        
        Ok(repos.into_iter().map(|r| ProjectRepoWithName {
            id: r.id,
            project_id: r.project_id,
            repo_id: r.repo_id,
            repo_name: r.repo_name,
            setup_script: r.setup_script,
            cleanup_script: r.cleanup_script,
            copy_files: r.copy_files,
            parallel_setup_script: r.parallel_setup_script,
        }).collect())
    }

    /// Find Repo details for a project
    pub async fn find_repos_for_project(
        _pool: &PgPool,
        _project_id: Uuid,
    ) -> Result<Vec<Repo>, sqlx::Error> {
        Ok(vec![])
    }

    /// Find specific project repo
    pub async fn find_by_project_and_repo(
        pool: &PgPool,
        project_id: Uuid,
        repo_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let repos = Self::find_by_project_id(pool, project_id).await?;
        Ok(repos.into_iter().find(|r| r.repo_id == repo_id))
    }

    /// Add repo to project
    pub async fn add_repo_to_project(
        pool: &PgPool,
        project_id: Uuid,
        repo_path: &str,
        repo_name: &str,
    ) -> Result<Repo, ProjectRepoError> {
        let repo = Repo::find_or_create(pool, Path::new(repo_path), repo_name).await
            .map_err(ProjectRepoError::Database)?;

        if Self::find_by_project_and_repo(pool, project_id, repo.id)
            .await?
            .is_some()
        {
            return Err(ProjectRepoError::AlreadyExists);
        }

        let project_repo = Self {
            id: Uuid::new_v4(),
            project_id,
            repo_id: repo.id,
            repo_name: repo.name.clone(),
            setup_script: None,
            cleanup_script: None,
            copy_files: None,
            parallel_setup_script: false,
        };

        let mut repos = Self::find_by_project_id(pool, project_id).await?;
        repos.push(project_repo);

        let repos_json = serde_json::json!({ "repos": repos });
        sqlx::query(
            r#"
            UPDATE projects 
            SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&repos_json)
        .bind(project_id)
        .execute(pool)
        .await?;

        Ok(repo)
    }

    /// Remove repo from project
    pub async fn remove_repo_from_project(
        pool: &PgPool,
        project_id: Uuid,
        repo_id: Uuid,
    ) -> Result<(), ProjectRepoError> {
        let mut repos = Self::find_by_project_id(pool, project_id).await?;
        
        let initial_len = repos.len();
        repos.retain(|r| r.repo_id != repo_id);
        
        if repos.len() == initial_len {
            return Err(ProjectRepoError::NotFound);
        }

        let repos_json = serde_json::json!({ "repos": repos });
        sqlx::query(
            r#"
            UPDATE projects 
            SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&repos_json)
        .bind(project_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Create project repo entry
    pub async fn create(
        _pool: &PgPool,
        _project_id: Uuid,
        _repo_id: Uuid,
    ) -> Result<Self, sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }

    /// Update project repo
    pub async fn update(
        pool: &PgPool,
        project_id: Uuid,
        repo_id: Uuid,
        payload: &UpdateProjectRepo,
    ) -> Result<Self, ProjectRepoError> {
        let mut repos = Self::find_by_project_id(pool, project_id).await?;
        
        let repo = repos
            .iter_mut()
            .find(|r| r.repo_id == repo_id)
            .ok_or(ProjectRepoError::NotFound)?;

        if let Some(setup_script) = &payload.setup_script {
            repo.setup_script = Some(setup_script.clone());
        }
        if let Some(cleanup_script) = &payload.cleanup_script {
            repo.cleanup_script = Some(cleanup_script.clone());
        }
        if let Some(copy_files) = &payload.copy_files {
            repo.copy_files = Some(copy_files.clone());
        }
        if let Some(parallel) = payload.parallel_setup_script {
            repo.parallel_setup_script = parallel;
        }

        let updated_repo = repo.clone();

        let repos_json = serde_json::json!({ "repos": repos });
        sqlx::query(
            r#"
            UPDATE projects 
            SET data = COALESCE(data, '{}'::jsonb) || $1::jsonb,
                updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(&repos_json)
        .bind(project_id)
        .execute(pool)
        .await?;

        Ok(updated_repo)
    }
}
