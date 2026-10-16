import { describe, expect, test } from "bun:test";
import { SCENARIOS, decodeTokenPayload, maskKeys, tamperSignature } from "./scenarios";

describe("tamperSignature", () => {
  const url = "/media/1800000000/0abc12/clip/stream.mpd";

  test("changes only the signature so the server sees a forged token, not a missing file", () => {
    const tampered = tamperSignature(url);
    expect(tampered).not.toBe(url);
    const [a, b] = [url.split("/"), tampered.split("/")];
    expect(b[2]).toBe(a[2]);
    expect(b[4]).toBe(a[4]);
    expect(b[5]).toBe(a[5]);
    expect(b[3].length).toBe(a[3].length);
  });

  test("leaves non signed urls untouched", () => {
    expect(tamperSignature("/api/session")).toBe("/api/session");
  });
});

describe("decodeTokenPayload", () => {
  test("reads base64url claims so the UI shows what the server will trust", () => {
    const payload = btoa(JSON.stringify({ sub: "alice", ent: ["clip"], exp: 1 })).replace(/=+$/, "").replace(/\+/g, "-").replace(/\//g, "_");
    expect(decodeTokenPayload(`h.${payload}.s`)).toEqual({ sub: "alice", ent: ["clip"], exp: 1 });
  });
});

describe("maskKeys", () => {
  test("never renders the full content key on screen", () => {
    const masked = maskKeys(JSON.stringify({ keys: [{ kty: "oct", kid: "abc", k: "SECRETKEYMATERIAL" }], type: "temporary" }));
    expect(masked).not.toContain("SECRETKEYMATERIAL");
    expect(masked).toContain("SECR...");
  });
});

describe("SCENARIOS", () => {
  test("each failure mode toggles exactly one control away from the happy path", () => {
    const happy = SCENARIOS.find((s) => s.id === "valid")!;
    const flags = (s: typeof happy) => [!s.entitled, s.sessionExpired, s.urlExpired, s.tamper].filter(Boolean).length;
    expect(flags(happy)).toBe(0);
    for (const s of SCENARIOS.filter((x) => x.id !== "valid")) expect(flags(s)).toBe(1);
  });
});
