import { useMutation } from '@tanstack/react-query';

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

const handleResponse = async <T,>(response: Response): Promise<T> => {
  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || `HTTP ${response.status}`);
  }
  const data = await response.json();
  if (!data.success) {
    throw new Error(data.message || 'Request failed');
  }
  return data.data;
};

export function useLocalAuthMutations() {
  const loginMutation = useMutation({
    mutationKey: ['auth', 'local', 'login'],
    mutationFn: async (credentials: LocalLoginCredentials) => {
      const response = await fetch('/api/auth/local/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(credentials),
      });
      return handleResponse<LocalLoginResponse>(response);
    },
  });

  const statusMutation = useMutation({
    mutationKey: ['auth', 'local', 'status'],
    mutationFn: async () => {
      const response = await fetch('/api/auth/local/status');
      return handleResponse<LocalAuthStatusResponse>(response);
    },
  });

  return {
    login: loginMutation,
    status: statusMutation,
  };
}