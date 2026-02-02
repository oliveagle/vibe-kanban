import { useCallback } from 'react';
import { Github, Chrome, User } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useAuthMutations } from '@/hooks/auth/useAuthMutations';
import { useLocalAuthMutations } from '@/hooks/auth/useLocalAuthMutations';
import { LocalLoginDialog } from '@/components/dialogs/auth/LocalLoginDialog';
import { useState, useEffect } from 'react';
import { useAuth } from '@/hooks';
import { useUserSystem } from '@/components/ConfigProvider';
import { Loader } from '@/components/ui/loader';

export function LandingPage() {
  const [showLocalLogin, setShowLocalLogin] = useState(false);
  const [localAuthEnabled, setLocalAuthEnabled] = useState(false);
  const { isSignedIn, isLoaded } = useAuth();
  const { reloadSystem } = useUserSystem();
  const { status } = useLocalAuthMutations();

  const { initHandoff } = useAuthMutations({
    onInitSuccess: (data) => {
      const width = 600;
      const height = 700;
      const left = window.screenX + (window.outerWidth - width) / 2;
      const top = window.screenY + (window.outerHeight - height) / 2;

      window.open(
        data.authorize_url,
        'oauth-popup',
        `width=${width},height=${height},left=${left},top=${top},popup=yes,noopener=yes`
      );
    },
  });

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const result = await status.mutateAsync();
        setLocalAuthEnabled(result.enabled);
      } catch (error) {
        setLocalAuthEnabled(false);
      }
    };
    checkStatus();
  }, []);

  const handleGitHubLogin = useCallback(() => {
    const returnTo = `${window.location.origin}/api/auth/handoff/complete`;
    initHandoff.mutate({ provider: 'github', returnTo });
  }, [initHandoff]);

  const handleGoogleLogin = useCallback(() => {
    const returnTo = `${window.location.origin}/api/auth/handoff/complete`;
    initHandoff.mutate({ provider: 'google', returnTo });
  }, [initHandoff]);

  if (!isLoaded) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <Loader message="Loading..." size={32} />
      </div>
    );
  }

  if (isSignedIn) {
    return null;
  }

  return (
    <div className="min-h-screen bg-background flex flex-col items-center justify-center p-4">
      <div className="max-w-md w-full space-y-8 text-center">
        <div className="space-y-2">
          <h1 className="text-3xl font-bold tracking-tight text-foreground">
            Vibe Kanban
          </h1>
          <p className="text-muted-foreground text-lg">
            登录以加入组织并与团队共享任务
          </p>
        </div>

        <div className="space-y-3">
          <Button
            variant="outline"
            className="w-full h-12 flex items-center justify-center gap-3"
            onClick={handleGitHubLogin}
            disabled={initHandoff.isPending}
          >
            <Github className="h-5 w-5" />
            <span>使用 GitHub 继续</span>
          </Button>

          <Button
            variant="outline"
            className="w-full h-12 flex items-center justify-center gap-3"
            onClick={handleGoogleLogin}
            disabled={initHandoff.isPending}
          >
            <Chrome className="h-5 w-5" />
            <span>使用 Google 继续</span>
          </Button>

          {localAuthEnabled && (
            <Button
              variant="outline"
              className="w-full h-12 flex items-center justify-center gap-3"
              onClick={() => setShowLocalLogin(true)}
            >
              <User className="h-5 w-5" />
              <span>使用用户名密码登录</span>
            </Button>
          )}
        </div>

        <p className="text-sm text-muted-foreground">
          登录即表示您同意我们的服务条款
        </p>
      </div>

      <LocalLoginDialog
        isOpen={showLocalLogin}
        onOpenChange={setShowLocalLogin}
        onLoginSuccess={() => {
          reloadSystem();
        }}
      />
    </div>
  );
}
