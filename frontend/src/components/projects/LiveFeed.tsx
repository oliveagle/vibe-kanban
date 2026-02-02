import { useNavigate } from 'react-router-dom';
import { Play, Clock, ArrowRight } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

import { useLiveFeed } from '@/hooks/useLiveFeed';
import { TaskStatus } from 'shared/types';

function getStatusColor(status: TaskStatus): string {
  switch (status) {
    case 'inprogress':
      return 'bg-blue-500';
    case 'todo':
      return 'bg-gray-400';
    case 'inreview':
      return 'bg-yellow-500';
    case 'done':
      return 'bg-green-500';
    case 'cancelled':
      return 'bg-red-500';
    default:
      return 'bg-gray-400';
  }
}

function getStatusLabel(status: TaskStatus): string {
  switch (status) {
    case 'inprogress':
      return '进行中';
    case 'todo':
      return '待办';
    case 'inreview':
      return '审核中';
    case 'done':
      return '已完成';
    case 'cancelled':
      return '已取消';
    default:
      return status;
  }
}

interface TaskItemProps {
  title: string;
  status: TaskStatus;
  projectName: string;
  projectId: string;
  hasInProgressAttempt?: boolean;
}

function TaskItem({ title, status, projectName, projectId, hasInProgressAttempt }: TaskItemProps) {
  const navigate = useNavigate();

  const handleClick = () => {
    navigate(`/projects/${projectId}/tasks`);
  };

  return (
    <div
      onClick={handleClick}
      className="p-3 border-b last:border-b-0 cursor-pointer hover:bg-muted/50 transition-colors group"
    >
      <div className="flex items-start gap-3">
        <div className={`mt-1 w-2 h-2 rounded-full ${getStatusColor(status)} flex-shrink-0`} />
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate group-hover:text-primary transition-colors">
            {title}
          </p>
          <div className="flex items-center gap-2 mt-1">
            <Badge variant="outline" className="text-xs">
              {projectName}
            </Badge>
            <Badge 
              variant={status === 'inprogress' ? 'default' : 'secondary'} 
              className="text-xs"
            >
              {getStatusLabel(status)}
            </Badge>
            {hasInProgressAttempt && (
              <Badge variant="default" className="text-xs bg-blue-500">
                <Play className="w-3 h-3 mr-1" />
                执行中
              </Badge>
            )}
          </div>
        </div>
        <ArrowRight className="w-4 h-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
      </div>
    </div>
  );
}

export function LiveFeed() {
  const { doingTasks, inReviewTasks, isLoading } = useLiveFeed();
  const allTasks = [...doingTasks, ...inReviewTasks];

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="pb-3">
        <CardTitle className="text-lg flex items-center gap-2">
          <Clock className="w-5 h-5 text-blue-500" />
          Live Feed
          {allTasks.length > 0 && (
            <Badge variant="secondary" className="ml-2">
              {allTasks.length}
            </Badge>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        {isLoading ? (
          <div className="p-4 text-center text-muted-foreground">
            加载中...
          </div>
        ) : allTasks.length === 0 ? (
          <div className="p-4 text-center text-muted-foreground">
            <p className="text-sm">暂无进行中的任务</p>
            <p className="text-xs mt-1">创建新任务后将在此显示</p>
          </div>
        ) : (
          <div className="h-[calc(100vh-200px)] overflow-y-auto">
            {doingTasks.length > 0 && (
              <div className="mb-4">
                <div className="px-4 py-2 bg-blue-50 dark:bg-blue-900/20">
                  <p className="text-xs font-medium text-blue-600 dark:text-blue-400">
                    Doing ({doingTasks.length})
                  </p>
                </div>
                {doingTasks.map((task) => (
                  <TaskItem
                    key={task.id}
                    title={task.title}
                    status={task.status}
                    projectName={task.projectName}
                    projectId={task.projectId}
                    hasInProgressAttempt={task.has_in_progress_attempt}
                  />
                ))}
              </div>
            )}
            {inReviewTasks.length > 0 && (
              <div>
                <div className="px-4 py-2 bg-yellow-50 dark:bg-yellow-900/20">
                  <p className="text-xs font-medium text-yellow-600 dark:text-yellow-400">
                    In Review ({inReviewTasks.length})
                  </p>
                </div>
                {inReviewTasks.map((task) => (
                  <TaskItem
                    key={task.id}
                    title={task.title}
                    status={task.status}
                    projectName={task.projectName}
                    projectId={task.projectId}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
