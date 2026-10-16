import type { ClientMsg, ServerMsg } from "../types/protocol";

export interface SocketHandlers {
  onOpen: () => void;
  onMessage: (msg: ServerMsg) => void;
  onClose: () => void;
}

export interface PartySocket {
  send: (msg: ClientMsg) => void;
  close: () => void;
}

export function socketUrl(): string {
  const scheme = window.location.protocol === "https:" ? "wss" : "ws";
  return `${scheme}://${window.location.host}/ws`;
}

export function openSocket(url: string, handlers: SocketHandlers): PartySocket {
  const ws = new WebSocket(url);
  ws.onopen = handlers.onOpen;
  ws.onclose = handlers.onClose;
  ws.onmessage = (event: MessageEvent<string>) => handlers.onMessage(JSON.parse(event.data) as ServerMsg);
  return {
    send: (msg) => {
      if (ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg));
    },
    close: () => ws.close(),
  };
}
