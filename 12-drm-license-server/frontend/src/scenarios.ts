import type { Scenario } from "./types";

export const SCENARIOS: Scenario[] = [
  {
    id: "valid",
    title: "Play (entitled)",
    description: "Fresh session with entitlement and a freshly signed URL.",
    entitled: true,
    sessionExpired: false,
    urlExpired: false,
    tamper: false,
    expect: "License granted, video decrypts and plays",
  },
  {
    id: "expired-url",
    title: "Expired URL token",
    description: "The CDN token expiry is already in the past.",
    entitled: true,
    sessionExpired: false,
    urlExpired: true,
    tamper: false,
    expect: "Segment server answers 403 expired on the manifest",
  },
  {
    id: "tampered-url",
    title: "Tampered URL signature",
    description: "One hex char of the URL HMAC is flipped in the browser.",
    entitled: true,
    sessionExpired: false,
    urlExpired: false,
    tamper: true,
    expect: "Segment server answers 403 bad_signature",
  },
  {
    id: "no-entitlement",
    title: "No entitlement",
    description: "Valid session without rights to the title.",
    entitled: false,
    sessionExpired: false,
    urlExpired: false,
    tamper: false,
    expect: "Ciphertext downloads, license server answers 403, nothing decrypts",
  },
  {
    id: "expired-session",
    title: "Expired session",
    description: "Session token already expired before playback starts.",
    entitled: true,
    sessionExpired: true,
    urlExpired: false,
    tamper: false,
    expect: "Playback API refuses to sign a URL",
  },
];

export function tamperSignature(url: string): string {
  const parts = url.split("/");
  if (parts.length < 6 || parts[1] !== "media") return url;
  const sig = parts[3];
  const flipped = sig[0] === "0" ? "1" : "0";
  parts[3] = flipped + sig.slice(1);
  return parts.join("/");
}

export function decodeTokenPayload(token: string): unknown {
  const payload = token.split(".")[1] ?? "";
  const base64 = payload.replace(/-/g, "+").replace(/_/g, "/");
  return JSON.parse(atob(base64 + "=".repeat((4 - (base64.length % 4)) % 4)));
}

export function maskKeys(licenseJson: string): string {
  try {
    const parsed = JSON.parse(licenseJson);
    if (Array.isArray(parsed.keys)) {
      parsed.keys = parsed.keys.map((k: { k?: string }) => ({ ...k, k: k.k ? `${k.k.slice(0, 4)}...` : k.k }));
    }
    return JSON.stringify(parsed, null, 2);
  } catch {
    return licenseJson;
  }
}
