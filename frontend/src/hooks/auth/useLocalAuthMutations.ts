import { useMutation } from '@tanstack/react-query';
import { handleApiResponse } from '@/lib/api';

interface LocalLoginCredentials {
  username: string;
  password: string;
}

interface LocalLoginResponse {
  access_token: string;
  expires_at: string;
  user_id: string;
  username: string;
}

interface LocalAuthStatusResponse {
  enabled: boolean;
  default_username: string;
  message: string;
}

const makeRequest = async (url: string, options: RequestInit = {}) => {
  const headers = new Headers(options.headers ?? {});
  if (!headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  return fetch(url, {
    ...options,
    headers,
  });
};

export function useLocalAuthMutations() {
  const loginMutation = useMutation({
    mutationKey: ['auth', 'local', 'login'],
    mutationFn: async (credentials: LocalLoginCredentials) => {
      const response = await makeRequest('/api/auth/local/login', {
        method: 'POST',
        body: JSON.stringify(credentials),
      });
      
      return handleApiResponse<LocalLoginResponse>(response);
    },
  });

  const statusMutation = useMutation({
    mutationKey: ['auth', 'local', 'status'],
    mutationFn: async () => {
      const response = await makeRequest('/api/auth/local/status', {
        method: 'GET',
      });
      
      return handleApiResponse<LocalAuthStatusResponse>(response);
    },
  });

  return {
    login: loginMutation,
    status: statusMutation,
  };
}