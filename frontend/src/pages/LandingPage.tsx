import { useEffect, useState } from 'react';
import { Github, Chrome, User } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

import { useLocalAuthMutations } from '@/hooks/auth/useLocalAuthMutations';
import { LocalLoginDialog } from '@/components/dialogs/auth/LocalLoginDialog';
import { useAuth } from '@/hooks';
import { useUserSystem } from '@/components/ConfigProvider';
import { Loader } from '@/components/ui/loader';

export function LandingPage() {
  const [showLocalLogin, setShowLocalLogin] = useState(false);
  const [localAuthEnabled, setLocalAuthEnabled] = useState(false);
  const { isSignedIn, isLoaded } = useAuth();
  const { reloadSystem } = useUserSystem();
  const { status } = useLocalAuthMutations();

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
      <Card className="w-full max-w-md space-y-6">
        <CardHeader>
          <CardTitle className="text-center text-2xl font-bold">
            Vibe Kanban
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-4">
          <p className="text-center text-muted-foreground text-lg mb-4">
            登录以加入组织并与团队共享任务
          </p>

          <div className="space-y-3">
            <button
              onClick={() => {
                const returnTo = `${window.location.origin}/api/auth/handoff/complete`;
                const width = 600;
                const height = 700;
                const left = window.screenX + (window.outerWidth - width) / 2;
                const top = window.screenY + (window.outerHeight - height) / 2;

                window.open(
                  `https://github.com/login/oauth/authorize?client_id=Ov23liuWU1aV2A9W4G7&redirect_uri=${encodeURIComponent(returnTo)}&scope=user:email`,
                  'oauth-popup',
                  `width=${width},height=${height},left=${left},top=${top},popup=yes,noopener=yes`
                );
              }}
              className="w-full h-14 flex items-center justify-center gap-3 bg-white border border-gray-300 hover:bg-gray-50 transition-colors rounded-lg"
            >
              <Github className="h-6 w-6 text-gray-700" />
              <span className="text-lg font-medium text-gray-900">使用 GitHub 继续</span>
            </button>

            <button
              onClick={() => {
                const returnTo = `${window.location.origin}/api/auth/handoff/complete`;
                const width = 600;
                const height = 700;
                const left = window.screenX + (window.outerWidth - width) / 2;
                const top = window.screenY + (window.outerHeight - height) / 2;

                window.open(
                  `https://accounts.google.com/o/oauth2/v2/auth?client_id=86074196785563-7p8h2t7g8k3t1m16l6q7&redirect_uri=${encodeURIComponent(returnTo)}&response_type=token&scope=https://www.googleapis.com/auth/userinfo.email`,
                  'oauth-popup',
                  `width=${width},height=${height},left=${left},top=${top},popup=yes,noopener=yes`
                );
              }}
              className="w-full h-14 flex items-center justify-center gap-3 bg-white border border-gray-300 hover:bg-gray-50 transition-colors rounded-lg"
            >
              <Chrome className="h-6 w-6 text-gray-700" />
              <span className="text-lg font-medium text-gray-900">使用 Google 继续</span>
            </button>

            {localAuthEnabled && (
              <button
                onClick={() => setShowLocalLogin(true)}
                className="w-full h-14 flex items-center justify-center gap-3 bg-white border border-gray-300 hover:bg-gray-50 transition-colors rounded-lg"
              >
                <User className="h-6 w-6 text-gray-700" />
                <span className="text-lg font-medium text-gray-900">使用用户名密码登录</span>
              </button>
            )}
          </div>

          <p className="text-center text-xs text-muted-foreground mt-4">
            登录即表示您同意我们的服务条款
          </p>
        </CardContent>

        <LocalLoginDialog
          isOpen={showLocalLogin}
          onOpenChange={setShowLocalLogin}
          onLoginSuccess={() => {
            reloadSystem();
          }}
        />
      </Card>
    </div>
  );
}
