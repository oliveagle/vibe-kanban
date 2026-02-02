import { useState, useEffect } from 'react';
import { GitBranch, Server, Monitor, Hash } from 'lucide-react';

const FRONTEND_BUILD_HASH = 'a1b2c3d';

interface VersionInfo {
  version: string;
  frontendHash: string;
  backendStatus: 'connected' | 'disconnected' | 'checking';
  backendVersion?: string;
  imageHash?: string;
}

export function VersionFooter() {
  const [info, setInfo] = useState<VersionInfo>({
    version: '0.0.0',
    frontendHash: FRONTEND_BUILD_HASH,
    backendStatus: 'checking',
  });

  useEffect(() => {
    fetch('/package.json')
      .then((res) => res.json())
      .then((data) => {
        setInfo((prev) => ({ ...prev, version: data.version || '0.0.0' }));
      })
      .catch(() => {
        setInfo((prev) => ({ ...prev, version: 'dev' }));
      });

    checkBackendStatus();
    const interval = setInterval(checkBackendStatus, 30000);

    return () => clearInterval(interval);
  }, []);

  const checkBackendStatus = async () => {
    try {
      const response = await fetch('/api/health', { 
        method: 'GET',
        headers: { 'Accept': 'application/json' }
      });
      if (response.ok) {
        const data = await response.json();
        setInfo((prev) => ({
          ...prev,
          backendStatus: 'connected',
          backendVersion: data.version,
          imageHash: data.image_hash,
        }));
      } else {
        setInfo((prev) => ({ ...prev, backendStatus: 'disconnected' }));
      }
    } catch {
      setInfo((prev) => ({ ...prev, backendStatus: 'disconnected' }));
    }
  };

  const getStatusColor = (status: VersionInfo['backendStatus']) => {
    switch (status) {
      case 'connected':
        return 'text-green-500';
      case 'disconnected':
        return 'text-red-500';
      case 'checking':
        return 'text-yellow-500';
    }
  };

  const getStatusText = (status: VersionInfo['backendStatus']) => {
    switch (status) {
      case 'connected':
        return '在线';
      case 'disconnected':
        return '离线';
      case 'checking':
        return '检查中';
    }
  };

  return (
    <div className="bg-muted/80 backdrop-blur-sm border-t border-border py-1 px-4 text-xs text-muted-foreground flex-shrink-0">
      <div className="flex items-center justify-between max-w-screen-2xl mx-auto">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1" title="应用版本">
            <GitBranch className="w-3 h-3" />
            <span>v{info.version}</span>
          </div>

          {info.imageHash && (
            <div className="flex items-center gap-1" title="镜像 Hash">
              <Hash className="w-3 h-3" />
              <span className="font-mono">{info.imageHash.slice(0, 8)}</span>
            </div>
          )}
        </div>

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1" title={`前端构建 Hash: ${info.frontendHash}`}>
            <Monitor className="w-3 h-3" />
            <span className="font-mono text-blue-500">{info.frontendHash}</span>
          </div>

          <div 
            className={`flex items-center gap-1 ${getStatusColor(info.backendStatus)}`}
            title={`后端状态: ${getStatusText(info.backendStatus)}${info.backendVersion ? ` (v${info.backendVersion})` : ''}`}
          >
            <Server className="w-3 h-3" />
            <span>{getStatusText(info.backendStatus)}</span>
            {info.backendVersion && (
              <span className="font-mono opacity-70">v{info.backendVersion}</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
