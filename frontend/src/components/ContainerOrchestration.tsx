import { useState, useEffect } from 'react';
import { 
  Play, 
  Square, 
  RotateCw, 
  Trash2, 
  Container,
  RefreshCw,
  ExternalLink
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Loader } from '@/components/ui/loader';
import { useToast } from '@/hooks/useToast';

interface PortMapping {
  host_port: string;
  container_port: string;
}

interface ContainerInfo {
  id: string;
  name: string;
  image: string;
  status: string;
  state: string;
  ports: PortMapping[];
  created: string;
}

export function ContainerOrchestration() {
  const [containers, setContainers] = useState<ContainerInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [actionInProgress, setActionInProgress] = useState<string | null>(null);
  const { toast } = useToast();

  const fetchContainers = async () => {
    try {
      setIsLoading(true);
      const response = await fetch('/api/orchestration/containers');
      const data = await response.json();
      if (data.success) {
        setContainers(data.data);
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: 'Failed to fetch containers',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchContainers();
    const interval = setInterval(fetchContainers, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleAction = async (containerId: string, action: string) => {
    try {
      setActionInProgress(`${action}-${containerId}`);
      const response = await fetch(`/api/orchestration/containers/${containerId}/${action}`, {
        method: 'POST',
      });
      const data = await response.json();
      
      if (data.success) {
        toast({
          title: 'Success',
          description: `Container ${action}ed successfully`,
        });
        fetchContainers();
      } else {
        throw new Error(data.message || 'Action failed');
      }
    } catch (error) {
      toast({
        title: 'Error',
        description: `Failed to ${action} container`,
        variant: 'destructive',
      });
    } finally {
      setActionInProgress(null);
    }
  };

  const getStateColor = (state: string) => {
    switch (state.toLowerCase()) {
      case 'running':
        return 'bg-green-500';
      case 'exited':
      case 'stopped':
        return 'bg-red-500';
      case 'paused':
        return 'bg-yellow-500';
      default:
        return 'bg-gray-500';
    }
  };

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  };

  if (isLoading && containers.length === 0) {
    return (
      <Card className="w-full">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Container className="w-5 h-5" />
            Container Orchestration
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex justify-center py-8">
            <Loader message="Loading containers..." />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Container className="w-5 h-5" />
            Container Orchestration
            <Badge variant="secondary">{containers.length}</Badge>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchContainers}
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          </Button>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {containers.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No containers found
            </div>
          ) : (
            containers.map((container) => (
              <div
                key={container.id}
                className="flex items-center justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <div className={`w-2 h-2 rounded-full ${getStateColor(container.state)}`} />
                    <span className="font-medium truncate">{container.name}</span>
                    <Badge variant="outline" className="text-xs">
                      {container.state}
                    </Badge>
                  </div>
                  <div className="text-sm text-muted-foreground mt-1">
                    {container.image}
                  </div>
                  {container.ports.length > 0 && (
                    <div className="flex flex-wrap gap-2 mt-2">
                      {container.ports.map((port, idx) => (
                        <Badge key={idx} variant="secondary" className="text-xs">
                          {port.host_port}:{port.container_port}
                        </Badge>
                      ))}
                    </div>
                  )}
                  <div className="text-xs text-muted-foreground mt-1">
                    Created: {formatDate(container.created)}
                  </div>
                </div>

                <div className="flex items-center gap-1 ml-4">
                  {container.state === 'running' ? (
                    <>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleAction(container.id, 'stop')}
                        disabled={actionInProgress === `stop-${container.id}`}
                        title="Stop"
                      >
                        <Square className="w-4 h-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleAction(container.id, 'restart')}
                        disabled={actionInProgress === `restart-${container.id}`}
                        title="Restart"
                      >
                        <RotateCw className="w-4 h-4" />
                      </Button>
                    </>
                  ) : (
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleAction(container.id, 'start')}
                      disabled={actionInProgress === `start-${container.id}`}
                      title="Start"
                    >
                      <Play className="w-4 h-4" />
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => handleAction(container.id, 'remove')}
                    disabled={actionInProgress === `remove-${container.id}`}
                    title="Remove"
                    className="text-destructive hover:text-destructive"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
