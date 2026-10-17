export interface Member {
  id: number;
  name: string;
  joinedAt: number;
}

export interface PlaybackState {
  playing: boolean;
  position: number;
  rate: number;
  at: number;
}

export interface RoomView {
  room: string;
  leader: number;
  members: Member[];
  state: PlaybackState;
}

export type ServerMsg =
  | { type: "welcome"; you: number }
  | { type: "pong"; t0: number; t1: number; t2: number }
  | { type: "room"; view: RoomView }
  | { type: "error"; message: string };

export type ClientMsg =
  | { type: "join"; room: string; name: string }
  | { type: "ping"; t0: number }
  | ({ type: "state" } & PlaybackState);
