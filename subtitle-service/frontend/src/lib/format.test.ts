import { describe, expect, test } from "bun:test";
import { extractCueStyles, fontsFor, formatBytes, formatCounts, formatPercent } from "./format";

describe("format", () => {
  test("bytes scale so the size table stays comparable across ASS, VTT and video", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.00 MB");
  });

  test("fidelity is shown as a whole percent", () => {
    expect(formatPercent(0.621)).toBe("62%");
  });

  test("tag counts read like ASS overrides so authors recognise what was lost", () => {
    expect(formatCounts({ frz: 1, blur: 2 })).toBe("\\frz x1, \\blur x2");
    expect(formatCounts({})).toBe("none");
  });

  test("only STYLE blocks are extracted, cue text never leaks into CSS", () => {
    const vtt = "WEBVTT\n\nSTYLE\n::cue(.a) { color: #FFF; }\n\n1\n00:00:01.000 --> 00:00:02.000\n::cue(.evil) {}\n";
    expect(extractCueStyles(vtt)).toBe("::cue(.a) { color: #FFF; }");
  });

  test("Japanese loads the CJK collection instead of Latin faces that lack its glyphs", () => {
    expect(fontsFor("ja-JP")).toEqual(["/media/fonts/NotoSansCJK-Regular.ttc"]);
    expect(fontsFor("pt-BR")).toContain("/media/fonts/NotoSans-Regular.ttf");
  });
});
