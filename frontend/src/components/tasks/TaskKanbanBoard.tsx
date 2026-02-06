import { memo, useState } from 'react';
import { useAuth } from '@/hooks';
import {
  type DragEndEvent,
  KanbanBoard,
  KanbanCards,
  KanbanHeader,
  KanbanProvider,
} from '@/components/ui/shadcn-io/kanban';

import { SelectableTaskCard } from './SelectableTaskCard';
import { BulkOperations } from './BulkOperations';
import type { TaskStatus, TaskWithAttemptStatus } from 'shared/types';
import { statusBoardColors, statusLabels } from '@/utils/statusLabels';
import type { SharedTaskRecord } from '@/hooks/useProjectTasks';
import { SharedTaskCard } from './SharedTaskCard';

export type KanbanColumnItem =
  | {
      type: 'task';
      task: TaskWithAttemptStatus;
      sharedTask?: SharedTaskRecord;
    }
  | {
      type: 'shared';
      task: SharedTaskRecord;
    };

export type KanbanColumns = Record<TaskStatus, KanbanColumnItem[]>;

interface TaskKanbanBoardProps {
  columns: KanbanColumns;
  onDragEnd: (event: DragEndEvent) => void;
  onViewTaskDetails: (task: TaskWithAttemptStatus) => void;
  onViewSharedTask?: (task: SharedTaskRecord) => void;
  selectedTaskId?: string;
  selectedSharedTaskId?: string | null;
  onCreateTask?: () => void;
  projectId: string;
  onTasksUpdated?: () => void;
}

function TaskKanbanBoard({
  columns,
  onDragEnd,
  onViewTaskDetails,
  onViewSharedTask,
  selectedTaskId,
  selectedSharedTaskId,
  onCreateTask,
  projectId,
  onTasksUpdated,
}: TaskKanbanBoardProps) {
  const { userId } = useAuth();
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedTasks, setSelectedTasks] = useState<TaskWithAttemptStatus[]>([]);

  // Flatten all tasks for bulk operations
  const allTasks = Object.values(columns).flat().filter(item => item.type === 'task').map(item => item.task);

  const handleTaskSelection = (task: TaskWithAttemptStatus, selected: boolean) => {
    if (selected) {
      setSelectedTasks(prev => [...prev, task]);
    } else {
      setSelectedTasks(prev => prev.filter(t => t.id !== task.id));
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between p-4 border-b bg-muted/30">
        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              setSelectionMode(!selectionMode);
              if (selectionMode) {
                setSelectedTasks([]);
              }
            }}
            className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
              selectionMode 
                ? 'bg-primary text-primary-foreground' 
                : 'bg-muted hover:bg-muted/80'
            }`}
          >
            {selectionMode ? 'Exit Selection' : 'Select Tasks'}
          </button>
          {selectionMode && selectedTasks.length > 0 && (
            <span className="text-sm text-muted-foreground">
              {selectedTasks.length} selected
            </span>
          )}
        </div>
        {onCreateTask && (
          <button
            onClick={onCreateTask}
            className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
          >
            Add Task
          </button>
        )}
      </div>

      {/* Bulk Operations */}
      {selectionMode && (
        <BulkOperations
          selectedTasks={selectedTasks}
          onSelectionChange={setSelectedTasks}
          availableTasks={allTasks}
          projectId={projectId}
          onTasksUpdated={onTasksUpdated}
        />
      )}

      {/* Kanban Board */}
      <KanbanProvider onDragEnd={onDragEnd}>
        <div className="flex-1 flex">
          {Object.entries(columns).map(([status, items]) => {
            const statusKey = status as TaskStatus;
            return (
              <KanbanBoard key={status} id={statusKey}>
                <KanbanHeader
                  name={statusLabels[statusKey]}
                  color={statusBoardColors[statusKey]}
                />
                <KanbanCards>
                  {items.map((item, index) => {
                    const isOwnTask =
                      item.type === 'task' &&
                      (!item.sharedTask?.assignee_user_id ||
                        !userId ||
                        item.sharedTask?.assignee_user_id === userId);

                    if (isOwnTask) {
                      return (
                        <SelectableTaskCard
                          key={item.task.id}
                          task={item.task}
                          index={index}
                          status={statusKey}
                          onViewDetails={onViewTaskDetails}
                          isOpen={selectedTaskId === item.task.id}
                          projectId={projectId}
                          sharedTask={item.sharedTask}
                          isSelected={selectedTasks.some(t => t.id === item.task.id)}
                          onSelectionChange={handleTaskSelection}
                          selectionMode={selectionMode}
                        />
                      );
                    }

                    const sharedTask =
                      item.type === 'shared' ? item.task : item.sharedTask!;

                    return (
                      <SharedTaskCard
                        key={`shared-${item.task.id}`}
                        task={sharedTask}
                        index={index}
                        status={statusKey}
                        isSelected={selectedSharedTaskId === item.task.id}
                        onViewDetails={onViewSharedTask}
                      />
                    );
                  })}
                </KanbanCards>
              </KanbanBoard>
            );
          })}
        </div>
      </KanbanProvider>
    </div>
  );
}

export default memo(TaskKanbanBoard);
