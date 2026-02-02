import { useState, useCallback, useEffect } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useLocalAuthMutations } from '@/hooks/auth/useLocalAuthMutations';

interface LocalLoginDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onLoginSuccess?: () => void;
}

export function LocalLoginDialog({
  isOpen,
  onOpenChange,
  onLoginSuccess,
}: LocalLoginDialogProps) {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isCheckingStatus, setIsCheckingStatus] = useState(false);
  const [localAuthEnabled, setLocalAuthEnabled] = useState(true); // Default to enabled
  const [error, setError] = useState<string | null>(null);
  const { login, status } = useLocalAuthMutations();

  // Check if local auth is enabled when dialog opens
  useEffect(() => {
    if (isOpen) {
      setIsCheckingStatus(true);
      status.mutateAsync()
        .then((response) => {
          setLocalAuthEnabled(response.enabled);
        })
        .catch((error) => {
          console.error('Failed to check local auth status:', error);
          // Default to enabled if check fails
          setLocalAuthEnabled(true);
        })
        .finally(() => {
          setIsCheckingStatus(false);
        });
    } else {
      // Reset when closing
      setLocalAuthEnabled(true);
      setError(null);
    }
  }, [isOpen, status]);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      
      try {
        await login.mutateAsync({ username, password });
        
        setError(null);
        onOpenChange(false);
        if (onLoginSuccess) {
          onLoginSuccess();
        }
      } catch (error) {
        console.error('Login failed:', error);
        setError('Invalid username or password. Please try again.');
      }
    },
    [username, password, onLoginSuccess, onOpenChange, login]
  );

  if (!localAuthEnabled && !isCheckingStatus) {
    return (
      <Dialog open={isOpen} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Local Authentication Disabled</DialogTitle>
          </DialogHeader>
          <div className="py-4 text-center text-muted-foreground">
            Local authentication is not enabled in this environment.
            Please use OAuth authentication instead.
          </div>
          <div className="flex justify-end">
            <Button onClick={() => onOpenChange(false)}>Close</Button>
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Login with Username and Password</DialogTitle>
        </DialogHeader>
        
        {isCheckingStatus ? (
          <div className="py-4 flex items-center justify-center">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {error && (
              <div className="text-red-500 text-sm p-2 bg-red-50 rounded-md">
                {error}
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="username">Username</Label>
              <Input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="Enter username"
                required
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter password"
                required
              />
            </div>
            
            <Button 
              type="submit" 
              className="w-full"
              disabled={login.isPending}
            >
              {login.isPending ? 'Logging in...' : 'Sign In'}
            </Button>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}