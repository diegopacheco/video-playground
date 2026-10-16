import type { TrackInfo } from "../types";

async function getText(path: string): Promise<string> {
  const res = await fetch(path);
  if (!res.ok) throw new Error(`${path} returned ${res.status}`);
  return res.text();
}

export async function fetchTracks(): Promise<TrackInfo[]> {
  return JSON.parse(await getText("/api/tracks")) as TrackInfo[];
}

export const fetchMaster = (): Promise<string> => getText("/hls/master.m3u8");

export const fetchVtt = (locale: string): Promise<string> => getText(vttUrl(locale));

export const vttUrl = (locale: string): string => `/api/tracks/${locale}/vtt`;
export const assUrl = (locale: string): string => `/api/tracks/${locale}/ass`;
export const burnedUrl = (locale: string): string => `/media/burned/${locale}.mp4`;
export const SOURCE_URL = "/media/source.mp4";
