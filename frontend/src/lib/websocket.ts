import { io, Socket } from 'socket.io-client';

let socket: Socket | null = null;

export const getSocket = (): Socket | null => socket;

export const connectSocket = (token?: string): Socket => {
  const WS_URL = import.meta.env.VITE_WS_URL || 'http://localhost:3001';
  socket = io(WS_URL.replace(/^http/, 'ws'), {
    auth: token ? { token } : {},
    transports: ['websocket'],
  });
  return socket;
};

export const disconnectSocket = () => {
  if (socket) {
    socket.disconnect();
    socket = null;
  }
};