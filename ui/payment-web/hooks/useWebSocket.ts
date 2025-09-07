// src/hooks/useWebSocket.ts
import { useEffect, useState, useCallback, useRef } from 'react';
import { useAuth } from '../contexts/AuthContext';

interface WebSocketMessage {
  type: string;
  [key: string]: any;
}

export const useWebSocket = (onMessage: (msg: WebSocketMessage) => void) => {
  const { user } = useAuth();
  const ws = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  const connect = useCallback(() => {
    if (!user) return;
    
    const wsUrl = `ws://localhost:3003?token=${localStorage.getItem('access_token')}`;
    ws.current = new WebSocket(wsUrl);

    ws.current.onopen = () => {
      console.log('WebSocket connected');
      setConnected(true);
    };

    ws.current.onmessage = (event) => {
      try {
        const msg: WebSocketMessage = JSON.parse(event.data);
        onMessage(msg);
      } catch (e) {
        console.error('Invalid WebSocket message', e);
      }
    };

    ws.current.onclose = () => {
      console.log('WebSocket disconnected');
      setConnected(false);
      // Reconnect after 3 seconds
      setTimeout(connect, 3000);
    };

    ws.current.onerror = (error) => {
      console.error('WebSocket error', error);
    };

    return () => {
      ws.current?.close();
    };
  }, [user, onMessage]);

  useEffect(() => {
    connect();
    return () => {
      ws.current?.close();
    };
  }, [connect]);

  return { connected };
};