import { useQuery } from '@tanstack/react-query';
import { oauthApi } from '@/lib/api';
import { useEffect } from 'react';
import { useAuth } from '@/hooks';

interface UseAuthStatusOptions {
  enabled: boolean;
}

export function useAuthStatus(options: UseAuthStatusOptions) {
  const query = useQuery({
    queryKey: ['auth', 'status'],
    queryFn: () => oauthApi.status(),
    enabled: options.enabled,
    refetchInterval: options.enabled ? 1000 : false,
    retry: 3,
    staleTime: 0, // Always fetch fresh data when enabled
  });

  const { isSignedIn } = useAuth();
  useEffect(() => {
    // Only refetch when sign-in status changes, not when query object changes
    query.refetch();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isSignedIn]);

  return query;
}
