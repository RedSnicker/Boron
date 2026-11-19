import axios from 'axios';

export const api = axios.create({ baseURL: import.meta.env.VITE_API_URL || '', headers: { 'Content-Type': 'application/json' } });
api.interceptors.request.use(config => {
  const token = localStorage.getItem('boron_token');
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});
api.interceptors.response.use(response => response, error => {
  const path = error.config?.url || '';
  const allow=String(error.response?.headers?.allow || '').toUpperCase();
  if (path === '/api/channels/unread' && error.response?.status === 405 && allow.includes('DELETE')) {
    window.dispatchEvent(new Event('boron-api-incompatible'));
  }
  if (error.response?.status === 401 && !path.includes('/api/auth/login') && !path.includes('/api/auth/register')) {
    localStorage.removeItem('boron_token');
    localStorage.removeItem('boron_user');
    window.dispatchEvent(new Event('boron-session-expired'));
  }
  return Promise.reject(error);
});
export const setAuthToken = (token: string | null) => {
  if (token) localStorage.setItem('boron_token', token);
  else { localStorage.removeItem('boron_token'); localStorage.removeItem('boron_user'); }
};
export const getAuthToken = () => localStorage.getItem('boron_token');
