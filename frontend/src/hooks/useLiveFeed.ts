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

  // Fetch tasks for all projects
  const projectQueries = useQuery({
    queryKey: ['live-feed-tasks'],
    queryFn: async () => {
      const allTasks: LiveTask[] = [];
      
      for (const project of projects) {
        try {
          const response = await fetch(`/api/projects/${project.id}/tasks`);
          if (!response.ok) continue;
          
          const data = await response.json();
          if (data.success && Array.isArray(data.data)) {
            const inProgressTasks = data.data.filter(
              (task: TaskWithAttemptStatus) => 
                task.status === 'inprogress' || 
                task.has_in_progress_attempt
            );
            
            inProgressTasks.forEach((task: TaskWithAttemptStatus) => {
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
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  const { inProgressTasks, todoTasks } = useMemo(() => {
    const tasks = projectQueries.data || [];
    
    // Sort by updated_at descending
    const sorted = tasks.sort((a, b) => {
      const aTime = new Date(a.updated_at || a.created_at).getTime();
      const bTime = new Date(b.updated_at || b.created_at).getTime();
      return bTime - aTime;
    });

    return {
      inProgressTasks: sorted.filter(t => t.status === 'inprogress'),
      todoTasks: sorted.filter(t => t.status === 'todo'),
    };
  }, [projectQueries.data]);

  return {
    inProgressTasks,
    todoTasks,
    isLoading: projectQueries.isLoading,
    error: projectQueries.error,
  };
}
