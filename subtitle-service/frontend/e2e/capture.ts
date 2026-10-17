import { chromium, type Page } from "playwright";

const base = process.env.UI_URL ?? "http://localhost:5106";
const out = process.env.OUT_DIR ?? "../printscreens";

async function compareAt(page: Page, locale: string, seconds: number, file: string): Promise<void> {
  await page.goto(`${base}/?locale=${locale}`);
  await page.waitForSelector("canvas.JASSUB", { state: "attached" });
  await page.waitForFunction(() => document.querySelector(".panel footer")?.textContent?.includes("fidelity"));
  await page.waitForFunction(() => [...document.querySelectorAll(".panel footer")].some((f) => f.textContent?.includes("ready in")));
  await page.evaluate(async (t) => {
    const videos = [...document.querySelectorAll("video")];
    await Promise.all(
      videos.map(
        (v) =>
          new Promise<void>((resolve) => {
            v.pause();
            v.addEventListener("seeked", () => resolve(), { once: true });
            v.currentTime = t;
          }),
      ),
    );
  }, seconds);
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${out}/${file}` });
  console.log(`saved ${file}`);
}

const browser = await chromium.launch({ channel: "chrome" });
const page = await browser.newPage({ viewport: { width: 1600, height: 760 } });
page.on("console", (m) => {
  if (m.type() === "error") console.log(`console error: ${m.text()}`);
});
page.on("response", (r) => {
  if (r.status() >= 400) console.log(`http ${r.status()} ${r.url()}`);
});
await compareAt(page, "en-US", 4.6, "compare-en-US.png");
await compareAt(page, "pt-BR", 11.2, "compare-pt-BR-karaoke.png");
await compareAt(page, "es-419", 7.0, "compare-es-419.png");
await compareAt(page, "ja-JP", 13.5, "compare-ja-JP.png");
await page.setViewportSize({ width: 1600, height: 900 });
await page.goto(`${base}/?view=report`);
await page.waitForSelector("table tbody tr");
await page.waitForFunction(() => document.querySelector("pre")?.textContent?.includes("EXT-X-MEDIA"));
await page.screenshot({ path: `${out}/report.png` });
console.log("saved report.png");
await browser.close();
