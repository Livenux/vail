import axios, { type AxiosError } from 'axios';

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      window.location.href = '/';
    }
    return Promise.reject(error);
  }
);

export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T | null;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: {
    id: number;
    username: string;
    nickname: string | null;
    avatar: string | null;
    email: string | null;
  };
}

export interface User {
  id: number;
  username: string;
  nickname: string | null;
  avatar: string | null;
  email: string | null;
}

export interface Host {
  id: number;
  name: string;
  hostname: string;
  port: number;
  username: string | null;
  credential_type: string | null;
  description: string | null;
  tags: unknown;
  status: number;
  create_time: string;
}

export interface UploadTask {
  id: number;
  task_no: string;
  host_id: number;
  file_size: number;
  uploaded_size: number;
  status: number;
  create_time: string;
}

export const authApi = {
  login: (data: LoginRequest) => api.post<ApiResponse<LoginResponse>>('/auth/login', data),
  logout: () => api.post<ApiResponse<void>>('/auth/logout', {}),
  me: () => api.get<ApiResponse<User>>('/auth/me'),
  refresh: (refresh_token: string) => api.post<ApiResponse<{ access_token: string; expires_in: number }>>('/auth/refresh', { refresh_token })
};

export const hostApi = {
  list: () => api.get<ApiResponse<Host[]>>('/hosts'),
  get: (id: number) => api.get<ApiResponse<Host>>(`/hosts/${id}`),
  create: (data: Partial<Host>) => api.post<ApiResponse<{ id: number }>>('/hosts', data),
  update: (id: number, data: Partial<Host>) => api.put<ApiResponse<void>>(`/hosts/${id}`, data),
  delete: (id: number) => api.delete<ApiResponse<void>>(`/hosts/${id}`)
};

export const sftpApi = {
  createTask: (data: { host_id: number; remote_path: string; file_name: string; file_size: number }) => 
    api.post<ApiResponse<{ id: number; task_no: string; status: number; uploaded_size: number; file_size: number }>>('/sftp/task', data),
  uploadChunk: async (taskId: number, chunkIndex: number, offset: number, content: Blob) => {
    const formData = new FormData();
    formData.append('task_id', String(taskId));
    formData.append('chunk_index', String(chunkIndex));
    formData.append('offset', String(offset));
    formData.append('content', content);
    return api.post<ApiResponse<{ task_id: number; chunk_index: number; uploaded_size: number }>>('/sftp/upload', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    });
  },
  complete: (taskId: number) => api.post<ApiResponse<{ task_id: number; status: string }>>('/sftp/complete', { task_id: taskId }),
  listTasks: () => api.get<ApiResponse<UploadTask[]>>('/sftp/tasks'),
  getTask: (id: number) => api.get<ApiResponse<UploadTask>>(`/sftp/tasks/${id}`)
};

export default api;
