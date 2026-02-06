-- Create notification function
CREATE OR REPLACE FUNCTION notify_table_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'table_changes',
        json_build_object(
            'table', TG_TABLE_NAME,
            'operation', TG_OP,
            'id', COALESCE(NEW.id, OLD.id)
        )::text
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create triggers for tables that need real-time updates
DROP TRIGGER IF EXISTS tasks_notify ON tasks;
CREATE TRIGGER tasks_notify
    AFTER INSERT OR UPDATE OR DELETE ON tasks
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS projects_notify ON projects;
CREATE TRIGGER projects_notify
    AFTER INSERT OR UPDATE OR DELETE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS workspaces_notify ON workspaces;
CREATE TRIGGER workspaces_notify
    AFTER INSERT OR UPDATE OR DELETE ON workspaces
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS execution_processes_notify ON execution_processes;
CREATE TRIGGER execution_processes_notify
    AFTER INSERT OR UPDATE OR DELETE ON execution_processes
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS sessions_notify ON sessions;
CREATE TRIGGER sessions_notify
    AFTER INSERT OR UPDATE OR DELETE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();
