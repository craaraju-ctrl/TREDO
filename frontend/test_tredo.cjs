const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({
    headless: "new",
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
    defaultViewport: { width: 1280, height: 800 }
  });
  const page = await browser.newPage();

  console.log("Navigating to http://localhost:3000/");
  await page.goto('http://localhost:3000/', { waitUntil: 'networkidle0' });

  console.log("Clicking Tredo tab...");
  await page.evaluate(() => {
    const tabs = Array.from(document.querySelectorAll('button, a, div'));
    const tredoTab = tabs.find(el => el.textContent && el.textContent.includes('Tredo'));
    if (tredoTab) tredoTab.click();
  });
  
  await new Promise(r => setTimeout(r, 2000));
  
  await page.screenshot({ path: 'screenshot.png' });
  console.log("Screenshot saved to screenshot.png");

  await browser.close();
})();
