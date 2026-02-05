import { useCallback, useEffect, useRef, useState } from 'react';
import { KanbanCard } from '@/components/ui/shadcn-io/kanban';
import { Link, Loader2, XCircle } from 'lucide-react';
import type { TaskWithAttemptStatus } from 'shared/types';
import { ActionsDropdown } from '@/components/ui/actions-dropdown';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { useNavigateWithSearch } from '@/hooks';
import { paths } from '@/lib/paths';
import { attemptsApi } from '@/lib/api';
import type { SharedTaskRecord } from '@/hooks/useProjectTasks';
import { TaskCardHeader } from './TaskCardHeader';
import { useTranslation } from 'react-i18next';
import { useAuth } from '@/hooks';

type Task = TaskWithAttemptStatus;

interface SelectableTaskCardProps {
  task: Task;
  index: number;
  status: string;
  onViewDetails: (task: Task) => void;
  isOpen?: boolean;
  projectId: string;
  sharedTask?: SharedTaskRecord;
  isSelected: boolean;
  onSelectionChange: (task: Task, selected: boolean) => void;
  selectionMode: boolean;
}

export function SelectableTaskCard({
  task,
  index,
  status,
  onViewDetails,
  isOpen,
  projectId,
  sharedTask,
  isSelected,
  onSelectionChange,
  selectionMode
}: SelectableTaskCardProps) {
  const { t } = useTranslation('tasks');
  const navigate = useNavigateWithSearch();
  const [isNavigatingToParent, setIsNavigatingToParent] = useState(false);
  const { isSignedIn } = useAuth();

  const handleClick = useCallback((e: React.MouseEvent) => {
    // Handle selection checkbox separately
    if ((e.target as HTMLElement).closest('[data-selection-checkbox]')) {
      return;
    }
    
    // Handle actions dropdown separately  
    if ((e.target as HTMLElement).closest('[data-actions-dropdown]')) {
      return;
    }

    if (selectionMode) {
      onSelectionChange(task, !isSelected);
    } else {
      onViewDetails(task);
    }
  }, [task, onViewDetails, selectionMode, isSelected, onSelectionChange]);

  const handleSelectionChange = useCallback((checked: boolean) => {
    onSelectionChange(task, checked);
  }, [task, onSelectionChange]);

  const handleParentClick = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      if (!task.parent_workspace_id || isNavigatingToParent) return;

      setIsNavigatingToParent(true);
      try {
        const parentAttempt = await attemptsApi.get(task.parent_workspace_id);
        navigate(
          paths.attempt(
            projectId,
            parentAttempt.task_id,
            task.parent_workspace_id
          )
        );
      } catch (error) {
        console.error('Failed to navigate to parent task attempt:', error);
        setIsNavigatingToParent(false);
      }
    },
    [task.parent_workspace_id, projectId, navigate, isNavigatingToParent]
  );

  const localRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isOpen || !localRef.current) return;
    const el = localRef.current;
    requestAnimationFrame(() => {
      el.scrollIntoView({
        block: 'center',
        inline: 'nearest',
        behavior: 'smooth',
      });
    });
  }, [isOpen]);

  return (
    <KanbanCard
      key={task.id}
      id={task.id}
      name={task.title}
      index={index}
      parent={status}
      onClick={handleClick}
      isOpen={isOpen}
      forwardedRef={localRef}
      dragDisabled={(!!sharedTask || !!task.shared_task_id) && !isSignedIn}
      className={
        (sharedTask || task.shared_task_id)
          ? 'relative overflow-hidden pl-5 before:absolute before:left-0 before:top-0 before:bottom-0 before:w-[3px] before:bg-card-foreground before:content-[""]'
          : undefined
      }
    >
      {/* Selection overlay */}
      {selectionMode && (
        <div className="absolute top-2 left-2 z-10">
          <Checkbox
            data-selection-checkbox
            checked={isSelected}
            onCheckedChange={handleSelectionChange}
            onClick={(e) => e.stopPropagation()}
            className="bg-background/80 backdrop-blur-sm border-2"
          />
        </div>
      )}

      <div className="flex flex-col gap-2">
        <TaskCardHeader
          title={task.title}
          avatar={
            sharedTask
              ? {
                  firstName: sharedTask.assignee_first_name ?? undefined,
                  lastName: sharedTask.assignee_last_name ?? undefined,
                  username: sharedTask.assignee_username ?? undefined,
                }
              : undefined
          }
          right={
            <>
              {task.has_in_progress_attempt && (
                <Loader2 className="h-4 w-4 animate-spin text-blue-500" />
              )}
              {task.last_attempt_failed && (
                <XCircle className="h-4 w-4 text-destructive" />
              )}
              {task.parent_workspace_id && (
                <Button
                  variant="icon"
                  onClick={handleParentClick}
                  onPointerDown={(e) => e.stopPropagation()}
                  onMouseDown={(e) => e.stopPropagation()}
                  disabled={isNavigatingToParent}
                  title={t('navigateToParent')}
                >
                  <Link className="h-4 w-4" />
                </Button>
              )}
              <div data-actions-dropdown>
                <ActionsDropdown task={task} sharedTask={sharedTask} />
              </div>
            </>
          }
        />
        {task.description && (
          <p className="text-sm text-secondary-foreground break-words">
            {task.description.length > 130
              ? `${task.description.substring(0, 130)}...`
              : task.description}
          </p>
        )}
      </div>

      {/* Selection indicator */}
      {selectionMode && isSelected && (
        <div className="absolute inset-0 ring-2 ring-primary ring-offset-2 rounded-lg pointer-events-none" />
      )}
    </KanbanCard>
  );
}