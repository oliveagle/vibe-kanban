import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Badge } from '@/components/ui/badge';
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuTrigger,
  DropdownMenuSeparator
} from '@/components/ui/dropdown-menu';
import { 
  Play, 
  CheckCircle, 
  XCircle, 
  Trash2, 
  MoreHorizontal,
  Loader2
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { tasksApi } from '@/lib/api';
import type { TaskStatus, TaskWithAttemptStatus } from 'shared/types';

interface BulkOperationsProps {
  selectedTasks: TaskWithAttemptStatus[];
  onSelectionChange: (tasks: TaskWithAttemptStatus[]) => void;
  availableTasks: TaskWithAttemptStatus[];
  projectId: string;
  onTasksUpdated?: () => void;
}

export function BulkOperations({
  selectedTasks,
  onSelectionChange,
  availableTasks,
  projectId,
  onTasksUpdated
}: BulkOperationsProps) {
  const { t } = useTranslation('tasks');
  const [isUpdating, setIsUpdating] = useState(false);

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      onSelectionChange(availableTasks);
    } else {
      onSelectionChange([]);
    }
  };

  const handleBulkStatusUpdate = async (status: TaskStatus) => {
    if (selectedTasks.length === 0) return;

    setIsUpdating(true);
    try {
      const taskIds = selectedTasks.map(task => task.id);
      await tasksApi.bulkUpdate(projectId, {
        task_ids: taskIds,
        status,
        title: null,
        description: null
      });
      
      alert(`Successfully updated ${selectedTasks.length} tasks to ${status}`);
      
      onSelectionChange([]);
      onTasksUpdated?.();
    } catch (error) {
      console.error('Failed to bulk update tasks:', error);
      alert('Failed to update tasks. Please try again.');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleBulkDelete = async () => {
    if (selectedTasks.length === 0) return;

    const confirmed = window.confirm(
      t('bulkDeleteConfirm', { count: selectedTasks.length })
    );
    if (!confirmed) return;

    setIsUpdating(true);
    try {
      const taskIds = selectedTasks.map(task => task.id);
      await tasksApi.bulkDelete(projectId, {
        task_ids: taskIds
      });
      
      alert(`Successfully deleted ${selectedTasks.length} tasks`);
      
      onSelectionChange([]);
      onTasksUpdated?.();
    } catch (error) {
      console.error('Failed to bulk delete tasks:', error);
      alert('Failed to delete tasks. Please try again.');
    } finally {
      setIsUpdating(false);
    }
  };

  const isAllSelected = availableTasks.length > 0 && selectedTasks.length === availableTasks.length;
  const isIndeterminate = selectedTasks.length > 0 && selectedTasks.length < availableTasks.length;
  const hasRunningProcesses = selectedTasks.some(task => task.has_in_progress_attempt);

  return (
    <div className="flex flex-col gap-3 p-4 border-b bg-muted/30">
      {/* Selection controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Checkbox
            checked={isAllSelected}
            onCheckedChange={handleSelectAll}
            data-indeterminate={isIndeterminate}
          />
          <span className="text-sm font-medium">
            {selectedTasks.length > 0 
              ? t('selectedCount', { count: selectedTasks.length })
              : t('selectTasks')
            }
          </span>
          {selectedTasks.length > 0 && (
            <Badge variant="secondary" className="text-xs">
              {selectedTasks.length}
            </Badge>
          )}
        </div>

        {selectedTasks.length > 0 && (
          <div className="flex items-center gap-2">
            {hasRunningProcesses && (
              <span className="text-xs text-amber-600">
                {t('someTasksRunning')}
              </span>
            )}
            
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button 
                  variant="outline" 
                  size="sm"
                  disabled={isUpdating || hasRunningProcesses}
                >
                  {isUpdating ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <MoreHorizontal className="h-4 w-4" />
                  )}
                  {t('bulkActions')}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => handleBulkStatusUpdate('inprogress')}>
                  <Play className="h-4 w-4 mr-2" />
                  {t('markInProgress')}
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleBulkStatusUpdate('done')}>
                  <CheckCircle className="h-4 w-4 mr-2" />
                  {t('markDone')}
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleBulkStatusUpdate('cancelled')}>
                  <XCircle className="h-4 w-4 mr-2" />
                  {t('markCancelled')}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem 
                  onClick={handleBulkDelete}
                  className="text-destructive"
                >
                  <Trash2 className="h-4 w-4 mr-2" />
                  {t('deleteSelected')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            <Button
              variant="ghost"
              size="sm"
              onClick={() => onSelectionChange([])}
            >
              {t('clearSelection')}
            </Button>
          </div>
        )}
      </div>

      {/* Quick action buttons */}
      {selectedTasks.length > 0 && (
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleBulkStatusUpdate('inprogress')}
            disabled={isUpdating || hasRunningProcesses}
          >
            <Play className="h-4 w-4 mr-1" />
            {t('start')}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleBulkStatusUpdate('done')}
            disabled={isUpdating || hasRunningProcesses}
          >
            <CheckCircle className="h-4 w-4 mr-1" />
            {t('done')}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleBulkStatusUpdate('cancelled')}
            disabled={isUpdating || hasRunningProcesses}
          >
            <XCircle className="h-4 w-4 mr-1" />
            {t('cancel')}
          </Button>
        </div>
      )}
    </div>
  );
}