import { useMemo } from 'react';
import { useProjects } from './useProjects';
import { useQuery } from '@tanstack/react-query';
import { TaskWithAttemptStatus } from 'shared/types';

interface LiveTask extends TaskWithAttemptStatus {
  projectName: string;
  projectId: string;
}

export function useLiveFeed() {
  const { projects } = useProjects();

  const projectQueries = useQuery({
    queryKey: ['live-feed-tasks'],
    queryFn: async () => {
      const allTasks: LiveTask[] = [];
      
      for (const project of projects) {
        try {
          const response = await fetch(`/api/tasks?project_id=${project.id}`);
          if (!response.ok) continue;
          
          const data = await response.json();
          if (data.success && Array.isArray(data.data)) {
            const activeTasks = data.data.filter(
              (task: TaskWithAttemptStatus) => 
                task.status === 'inprogress' || 
                task.status === 'inreview' ||
                task.status === 'todo' ||
                task.has_in_progress_attempt
            );
            
            activeTasks.forEach((task: TaskWithAttemptStatus) => {
              allTasks.push({
                ...task,
                projectName: project.name,
                projectId: project.id,
              });
            });
          }
        } catch (error) {
          console.error(`Failed to fetch tasks for project ${project.id}:`, error);
        }
      }
      
      return allTasks;
    },
    enabled: projects.length > 0,
    refetchInterval: 5000,
  });

  const { doingTasks, inReviewTasks } = useMemo(() => {
    const tasks = projectQueries.data || [];
    
    const sorted = tasks.sort((a, b) => {
      const aTime = new Date(a.updated_at || a.created_at).getTime();
      const bTime = new Date(b.updated_at || b.created_at).getTime();
      return bTime - aTime;
    });

    return {
      doingTasks: sorted.filter(t => t.status === 'inprogress' || t.has_in_progress_attempt),
      inReviewTasks: sorted.filter(t => t.status === 'inreview'),
    };
  }, [projectQueries.data]);

  return {
    doingTasks,
    inReviewTasks,
    isLoading: projectQueries.isLoading,
    error: projectQueries.error,
  };
}
