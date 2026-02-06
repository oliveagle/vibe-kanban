use server::mcp::task_server::{TaskServer, ProjectSummary, TaskSummary, McpContext, McpRepoContext};
use db::models::{
    project::Project,
    task::{TaskWithAttemptStatus, TaskStatus},
};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_task_server_new_with_timeout() {
    let server = TaskServer::new("http://localhost:3000");
    
    // Verify the server was created with proper configuration
    assert_eq!(server.base_url, "http://localhost:3000");
    assert!(server.context.is_none());
}

#[test]
fn test_task_server_url_construction() {
    let server = TaskServer::new("http://localhost:3000");
    
    assert_eq!(server.url("/api/tags"), "http://localhost:3000/api/tags");
    assert_eq!(server.url("api/tags"), "http://localhost:3000/api/tags");
    assert_eq!(server.url("/api/tags/"), "http://localhost:3000/api/tags/");
}

#[test]
fn test_task_server_url_with_trailing_slash() {
    let server = TaskServer::new("http://localhost:3000/");
    
    assert_eq!(server.url("/api/tags"), "http://localhost:3000/api/tags");
    assert_eq!(server.url("api/tags"), "http://localhost:3000/api/tags");
}

#[tokio::test]
async fn test_expand_tags_no_tags_in_text() {
    let server = TaskServer::new("http://localhost:3000");
    
    // Text without any @tags should return as-is
    let result = server.expand_tags("Hello world").await;
    assert_eq!(result, "Hello world");
    
    let result = server.expand_tags("No tags here").await;
    assert_eq!(result, "No tags here");
}

#[tokio::test]
async fn test_expand_tags_empty_text() {
    let server = TaskServer::new("http://localhost:3000");
    
    let result = server.expand_tags("").await;
    assert_eq!(result, "");
}

#[tokio::test]
async fn test_expand_tags_with_at_symbol_only() {
    let server = TaskServer::new("http://localhost:3000");
    
    // Just @ without a tag name should be ignored
    let result = server.expand_tags("Email me at @").await;
    assert_eq!(result, "Email me at @");
}

#[tokio::test]
async fn test_task_server_success_response() {
    let data = serde_json::json!({"test": "data"});
    let result = TaskServer::success(&data);
    
    assert!(result.is_ok());
    let call_result = result.unwrap();
    // The content should contain our data
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_task_server_error_response() {
    let result = TaskServer::err("Test error", Some("Details here"));
    
    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[tokio::test]
async fn test_task_server_error_without_details() {
    let result = TaskServer::err::<_, &str>("Test error", None);
    
    assert!(result.is_ok());
    let call_result = result.unwrap();
    assert!(!call_result.content.is_empty());
}

#[test]
fn test_project_summary_from_project() {
    let project = Project {
        id: Uuid::new_v4(),
        name: "Test Project".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let summary = ProjectSummary::from_project(project.clone());
    
    assert_eq!(summary.name, "Test Project");
    assert_eq!(summary.id, project.id.to_string());
}

#[test]
fn test_task_summary_from_task() {
    let task = TaskWithAttemptStatus {
        id: Uuid::new_v4(),
        title: "Test Task".to_string(),
        status: TaskStatus::InProgress,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        has_in_progress_attempt: true,
        last_attempt_failed: false,
    };
    
    let summary = TaskSummary::from_task_with_status(task.clone());
    
    assert_eq!(summary.title, "Test Task");
    assert_eq!(summary.id, task.id.to_string());
    assert_eq!(summary.status, "inprogress");
    assert_eq!(summary.has_in_progress_attempt, Some(true));
    assert_eq!(summary.last_attempt_failed, Some(false));
}

#[test]
fn test_task_summary_with_different_statuses() {
    for status in [
        TaskStatus::Todo,
        TaskStatus::InProgress,
        TaskStatus::InReview,
        TaskStatus::Done,
        TaskStatus::Cancelled,
    ] {
        let task = TaskWithAttemptStatus {
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            status,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            has_in_progress_attempt: false,
            last_attempt_failed: false,
        };
        
        let summary = TaskSummary::from_task_with_status(task);
        // Just verify it doesn't panic and produces valid string
        assert!(!summary.status.is_empty());
    }
}

#[test]
fn test_mcp_context_creation() {
    let context = McpContext {
        project_id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
        task_title: "Test Task".to_string(),
        workspace_id: Uuid::new_v4(),
        workspace_branch: "feature/test".to_string(),
        workspace_repos: vec![],
    };
    
    assert_eq!(context.task_title, "Test Task");
    assert_eq!(context.workspace_branch, "feature/test");
}

#[test]
fn test_mcp_repo_context_creation() {
    let repo = McpRepoContext {
        repo_id: Uuid::new_v4(),
        repo_name: "test-repo".to_string(),
        target_branch: "main".to_string(),
    };
    
    assert_eq!(repo.repo_name, "test-repo");
    assert_eq!(repo.target_branch, "main");
}

#[test]
fn test_mcp_context_with_repos() {
    let repo1 = McpRepoContext {
        repo_id: Uuid::new_v4(),
        repo_name: "repo1".to_string(),
        target_branch: "main".to_string(),
    };
    let repo2 = McpRepoContext {
        repo_id: Uuid::new_v4(),
        repo_name: "repo2".to_string(),
        target_branch: "develop".to_string(),
    };
    
    let context = McpContext {
        project_id: Uuid::new_v4(),
        task_id: Uuid::new_v4(),
        task_title: "Test".to_string(),
        workspace_id: Uuid::new_v4(),
        workspace_branch: "feature/test".to_string(),
        workspace_repos: vec![repo1, repo2],
    };
    
    assert_eq!(context.workspace_repos.len(), 2);
    assert_eq!(context.workspace_repos[0].repo_name, "repo1");
    assert_eq!(context.workspace_repos[1].target_branch, "develop");
}
