import { writable } from 'svelte/store';
import { authApi, type User } from './api';

interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  loading: boolean;
}

function createAuthStore() {
  const { subscribe, set, update } = writable<AuthState>({
    isAuthenticated: false,
    user: null,
    loading: true
  });

  return {
    subscribe,
    login: (token: string, refreshToken: string, user: User) => {
      localStorage.setItem('access_token', token);
      localStorage.setItem('refresh_token', refreshToken);
      update(state => ({
        ...state,
        isAuthenticated: true,
        user,
        loading: false
      }));
    },
    logout: async () => {
      try {
        await authApi.logout();
      } catch {
        // ignore
      }
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      set({
        isAuthenticated: false,
        user: null,
        loading: false
      });
    },
    setUser: (user: User) => {
      update(state => ({ ...state, user }));
    },
    setLoading: (loading: boolean) => {
      update(state => ({ ...state, loading }));
    }
  };
}

export const auth = createAuthStore();

export async function checkAuth() {
  const token = localStorage.getItem('access_token');
  if (!token) {
    auth.setLoading(false);
    return;
  }

  try {
    const res = await authApi.me();
    if (res.data.code === 200 && res.data.data) {
      auth.setUser(res.data.data);
      auth.setLoading(false);
    } else {
      auth.logout();
    }
  } catch {
    auth.logout();
  }
}
