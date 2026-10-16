import { chromium, type Page } from "playwright";
import { formatMs, summarize } from "../src/lib/stats";
import type { Mode, Sample, Summary } from "../src/types/latency";

const url = process.env.UI_URL ?? "http://localhost:5104";
const warmupMs = Number(process.env.WARMUP_MS ?? 10_000);
const measureMs = Number(process.env.MEASURE_MS ?? 30_000);
const shotsDir = process.env.SHOTS_DIR ?? "";

async function current(page: Page, mode: Mode): Promise<number | null> {
  return page.evaluate((m) => window.latencySummary?.[m]?.current ?? null, mode);
}

async function measure(page: Page, mode: Mode): Promise<Summary> {
  await page.click(`button[data-mode="${mode}"]`);
  await page.waitForTimeout(warmupMs);
  const samples: Sample[] = [];
  const end = Date.now() + measureMs;
  while (Date.now() < end) {
    const value = await current(page, mode);
    if (value !== null) samples.push({ at: Date.now(), latencyMs: value });
    await page.waitForTimeout(500);
  }
  if (shotsDir) await page.screenshot({ path: `${shotsDir}/${mode}.png` });
  return summarize(samples);
}

const browser = await chromium.launch({ channel: "chrome" });
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  await page.goto(url);
  const results: Record<Mode, Summary> = { ll: await measure(page, "ll"), regular: await measure(page, "regular") };
  for (const [mode, s] of Object.entries(results)) {
    console.log(
      `${mode.padEnd(8)} samples=${s.count} min=${formatMs(s.min)} avg=${formatMs(s.avg)} p50=${formatMs(s.p50)} p95=${formatMs(s.p95)}`,
    );
  }
} finally {
  await browser.close();
}
