export type Tier = "free" | "premium";

export interface UserProfile {
  id: string;
  name: string;
  region: string;
  tier: Tier;
}

export interface StitchedBreak {
  id: string;
  planned: number;
  replaced: number;
  filled: number;
  creatives: string[];
}

export interface StitchView {
  user: UserProfile;
  session: string;
  raw: string;
  stitched: string;
  breaks: StitchedBreak[];
}

export type AdEvent = "impression" | "firstQuartile" | "midpoint" | "thirdQuartile" | "complete";

export interface Beacon {
  seq: number;
  user: string;
  session: string;
  breakId: string;
  creative: string;
  event: AdEvent;
  atMs: number;
}

export type LineKind = "cue" | "discontinuity" | "ad" | "slate" | "content" | "info" | "header";
