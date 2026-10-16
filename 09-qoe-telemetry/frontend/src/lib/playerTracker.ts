import type { Beacon, BeaconEvent } from "../types/telemetry";

export class PlayerTracker {
  private requestedAt: number | null = null;
  private started = false;
  private stalledAt: number | null = null;
  private kbps: number | null = null;
  private lastBeat: number | null = null;
  private stallSinceBeat = 0;

  constructor(
    private readonly sessionId: string,
    private readonly title: string,
    private readonly cdn: string,
  ) {}

  private beacon(ts: number, event: BeaconEvent): Beacon {
    return { session_id: this.sessionId, title: this.title, cdn: this.cdn, ts, ...event };
  }

  requestPlay(ts: number): Beacon[] {
    if (this.requestedAt === null) this.requestedAt = ts;
    return [];
  }

  playing(ts: number): Beacon[] {
    if (!this.started && this.requestedAt !== null) {
      this.started = true;
      this.lastBeat = ts;
      return [this.beacon(ts, { kind: "startup", startup_ms: Math.max(0, Math.round(ts - this.requestedAt)) })];
    }
    if (this.stalledAt !== null) {
      const rebuffer_ms = Math.max(0, Math.round(ts - this.stalledAt));
      this.stallSinceBeat += Math.max(0, ts - Math.max(this.stalledAt, this.lastBeat ?? this.stalledAt));
      this.stalledAt = null;
      return [this.beacon(ts, { kind: "rebuffer_end", rebuffer_ms })];
    }
    return [];
  }

  waiting(ts: number): Beacon[] {
    if (!this.started || this.stalledAt !== null) return [];
    this.stalledAt = ts;
    return [this.beacon(ts, { kind: "rebuffer_start" })];
  }

  levelSwitched(ts: number, kbps: number): Beacon[] {
    const from = this.kbps;
    this.kbps = kbps;
    if (from === null || from === kbps) return [];
    return [this.beacon(ts, { kind: "bitrate_switch", from_kbps: from, to_kbps: kbps })];
  }

  error(ts: number, code: string): Beacon[] {
    return [this.beacon(ts, { kind: "error", code })];
  }

  heartbeat(ts: number, paused: boolean): Beacon[] {
    if (!this.started || this.lastBeat === null) return [];
    const elapsed = ts - this.lastBeat;
    const openStall = this.stalledAt !== null ? ts - Math.max(this.stalledAt, this.lastBeat) : 0;
    this.lastBeat = ts;
    const stalled = this.stallSinceBeat;
    this.stallSinceBeat = 0;
    if (paused) return [];
    const played_ms = Math.max(0, Math.round(elapsed - stalled - openStall));
    return [this.beacon(ts, { kind: "heartbeat", played_ms })];
  }
}
