import { chromium } from "playwright";

const base = process.env.UI_URL ?? "http://localhost:5106";
const seconds = Number(process.env.SECONDS ?? 12);
const locale = process.env.LOCALE ?? "en-US";
const runs = Number(process.env.RUNS ?? 3);

interface ProcessInfo {
  cpuTime: number;
}

const browser = await chromium.launch({ channel: "chrome" });
const cdp = await browser.newBrowserCDPSession();

async function cpuTotal(): Promise<number> {
  const info = (await cdp.send("SystemInfo.getProcessInfo")) as unknown as { processInfo: ProcessInfo[] };
  return info.processInfo.reduce((sum, p) => sum + p.cpuTime, 0);
}

async function measure(mode: string): Promise<number> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  await page.goto(`${base}/?solo=${mode}&locale=${locale}`);
  await page.waitForSelector("video");
  if (mode === "jassub") {
    await page.waitForFunction(() => document.querySelector(".panel footer")?.textContent?.includes("ready in"));
  }
  await page.waitForTimeout(1000);
  await page.click("button.primary");
  const before = await cpuTotal();
  await page.waitForTimeout(seconds * 1000);
  const after = await cpuTotal();
  await page.close();
  return after - before;
}

console.log(`mode,run,cpu_seconds_per_${seconds}s,cpu_percent_of_one_core`);
for (const mode of ["native", "jassub", "burned"]) {
  for (let run = 1; run <= runs; run++) {
    const cpu = await measure(mode);
    console.log(`${mode},${run},${cpu.toFixed(3)},${((cpu / seconds) * 100).toFixed(1)}`);
  }
}
await browser.close();
