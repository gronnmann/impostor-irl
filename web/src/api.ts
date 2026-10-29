import { useCallback, useEffect, useRef, useState } from 'react';
import type { CommandResult, Snapshot } from './types';

export async function sendCommand(command: Record<string, unknown>, token?: string): Promise<CommandResult> {
  const response = await fetch('/api/command', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command, ...(token ? { token } : {}) }) });
  const result = await response.json();
  if (!result.ok) throw new Error(result.error || 'Mission control could not complete that action.');
  return result;
}

export function useSnapshot(view: 'host' | 'player', token?: string, enabled = true) {
  const [state, setState] = useState<Snapshot | null>(null);
  const [online, setOnline] = useState(false);
  const generation = useRef(0);
  const refresh = useCallback(async () => {
    if (!enabled) return;
    const g = generation.current;
    try {
      const query = new URLSearchParams({ view, ...(token ? { token } : {}) });
      const response = await fetch(`/api/state?${query}`);
      if (!response.ok) throw new Error('Offline');
      const data = await response.json();
      if (g === generation.current) { setState(data); setOnline(true); }
    } catch { if (g === generation.current) setOnline(false); }
  }, [view, token, enabled]);
  useEffect(() => {
    generation.current++;
    setState(null);
    if (!enabled) { setOnline(false); return; }
    let stopped = false;
    let socket: WebSocket | undefined;
    let retry: ReturnType<typeof setTimeout>;
    const connect = () => {
      const query = new URLSearchParams({ view, ...(token ? { token } : {}) });
      socket = new WebSocket(`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/api/ws?${query}`);
      socket.onmessage = event => {
        if (stopped) return;
        try { const message = JSON.parse(event.data); if (message.type === 'state') { setState(message.state); setOnline(true); } } catch { /* polling can recover */ }
      };
      socket.onclose = () => { if (!stopped) retry = setTimeout(connect, 2500); };
      socket.onerror = () => socket?.close();
    };
    refresh(); connect();
    const polling = setInterval(refresh, 2000);
    return () => { stopped = true; clearInterval(polling); clearTimeout(retry); socket?.close(); };
  }, [refresh, view, token, enabled]);
  return { state, online, refresh };
}
