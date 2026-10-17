import { useCallback, useEffect, useRef, useState } from "react";
import { openSocket, socketUrl, type PartySocket } from "../api/socket";
import { bestSample, pushSample, sampleFrom, type ClockSample } from "../sync/clock";
import { localNow } from "../sync/localClock";
import type { PlaybackState, RoomView } from "../types/protocol";

const BURST_PINGS = 8;
const BURST_GAP_MS = 150;
const PING_EVERY_MS = 2000;

export interface Party {
  connected: boolean;
  you: number | null;
  view: RoomView | null;
  clock: ClockSample | null;
  samples: number;
  error: string | null;
  serverNow: () => number;
  sendState: (state: PlaybackState) => void;
}

export function useParty(room: string, name: string): Party {
  const socket = useRef<PartySocket | null>(null);
  const samples = useRef<ClockSample[]>([]);
  const offset = useRef(0);
  const [connected, setConnected] = useState(false);
  const [you, setYou] = useState<number | null>(null);
  const [view, setView] = useState<RoomView | null>(null);
  const [clock, setClock] = useState<ClockSample | null>(null);
  const [count, setCount] = useState(0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const timers: number[] = [];
    const ping = () => socket.current?.send({ type: "ping", t0: localNow() });
    const s = openSocket(socketUrl(), {
      onOpen: () => {
        setConnected(true);
        s.send({ type: "join", room, name });
        for (let i = 0; i < BURST_PINGS; i++) timers.push(window.setTimeout(ping, i * BURST_GAP_MS));
        timers.push(window.setInterval(ping, PING_EVERY_MS));
      },
      onClose: () => setConnected(false),
      onMessage: (msg) => {
        switch (msg.type) {
          case "welcome":
            setYou(msg.you);
            break;
          case "pong": {
            samples.current = pushSample(samples.current, sampleFrom(msg.t0, msg.t1, msg.t2, localNow()));
            const best = bestSample(samples.current);
            if (best) offset.current = best.offset;
            setClock(best);
            setCount(samples.current.length);
            break;
          }
          case "room":
            setView(msg.view);
            break;
          case "error":
            setError(msg.message);
            break;
        }
      },
    });
    socket.current = s;
    return () => {
      timers.forEach((t) => window.clearTimeout(t));
      timers.forEach((t) => window.clearInterval(t));
      s.close();
      socket.current = null;
    };
  }, [room, name]);

  const serverNow = useCallback(() => localNow() + offset.current, []);
  const sendState = useCallback((state: PlaybackState) => socket.current?.send({ type: "state", ...state }), []);

  return { connected, you, view, clock, samples: count, error, serverNow, sendState };
}
