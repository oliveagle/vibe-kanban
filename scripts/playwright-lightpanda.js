const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.connectOverCDP('ws://localhost:9222/');
  const context = await browser.newContext();
  const page = await context.newPage();
  
  await page.goto('http://localhost:3000/projects');
  await page.screenshot({ path: 'screenshot.png', fullPage: true });
  
  console.log('Screenshot saved to screenshot.png');
  
  await browser.close();
})();
