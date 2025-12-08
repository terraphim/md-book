#!/usr/bin/env node
/**
 * Record demo animations for md-book documentation
 * Creates animated WebP files from screenshots
 *
 * Usage: npx puppeteer browsers install chrome && node scripts/record-demos.mjs
 */

import puppeteer from 'puppeteer';
import { execSync, spawnSync } from 'child_process';
import { existsSync, mkdirSync, readdirSync, unlinkSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = join(__dirname, '..');
const GIF_DIR = join(PROJECT_ROOT, 'gif');
const FRAMES_DIR = join(PROJECT_ROOT, 'frames');

// Ensure directories exist
[GIF_DIR, FRAMES_DIR].forEach(dir => {
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
});

const BASE_URL = process.env.BASE_URL || 'http://localhost:3000';

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function clearFrames(prefix) {
  const files = readdirSync(FRAMES_DIR).filter(f => f.startsWith(prefix));
  files.forEach(f => unlinkSync(join(FRAMES_DIR, f)));
}

async function captureFrame(page, prefix, frameNum) {
  const path = join(FRAMES_DIR, `${prefix}_${String(frameNum).padStart(4, '0')}.png`);
  await page.screenshot({ path, type: 'png' });
  return path;
}

function framesToWebP(prefix, outputName, fps = 10) {
  const outputPath = join(GIF_DIR, `${outputName}.webp`);
  const inputPattern = join(FRAMES_DIR, `${prefix}_%04d.png`);

  console.log(`  Converting frames to ${outputPath}`);

  try {
    // Create animated WebP from PNG frames
    execSync(`ffmpeg -y -framerate ${fps} -i "${inputPattern}" -vf "scale=1280:-1:flags=lanczos" -vcodec libwebp -lossless 0 -compression_level 6 -q:v 75 -loop 0 -preset default -an "${outputPath}"`, {
      stdio: 'pipe'
    });
    console.log(`  ✅ Created: ${outputPath}`);

    // Clean up frames
    clearFrames(prefix);
    return outputPath;
  } catch (error) {
    console.error(`  ❌ Failed to convert: ${error.message}`);
    return null;
  }
}

async function recordResponsiveDemo(browser) {
  console.log('\n📱 Recording responsive layout demo...');
  const prefix = 'responsive';
  clearFrames(prefix);

  const page = await browser.newPage();
  let frameNum = 0;

  // Start at 2K resolution
  await page.setViewport({ width: 2560, height: 1440 });
  await page.goto(BASE_URL, { waitUntil: 'networkidle0' });
  await sleep(500);

  // Capture frames at different viewports
  const viewports = [
    { width: 2560, height: 1440, name: '2K', frames: 8 },
    { width: 1920, height: 1080, name: 'Full HD', frames: 8 },
    { width: 1440, height: 900, name: 'Laptop', frames: 8 },
    { width: 1024, height: 768, name: 'Tablet Landscape', frames: 8 },
    { width: 768, height: 1024, name: 'Tablet Portrait', frames: 8 },
    { width: 428, height: 926, name: 'iPhone Pro Max', frames: 8 },
    { width: 375, height: 667, name: 'iPhone SE', frames: 10 },
  ];

  for (const vp of viewports) {
    console.log(`  📐 ${vp.name} (${vp.width}x${vp.height})`);
    await page.setViewport({ width: vp.width, height: vp.height });
    await sleep(300);

    // Capture multiple frames at each size for smoother animation
    for (let i = 0; i < vp.frames; i++) {
      await captureFrame(page, prefix, frameNum++);
      await sleep(50);
    }
  }

  // Hold on mobile for a moment
  for (let i = 0; i < 5; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(100);
  }

  await page.close();
  return framesToWebP(prefix, 'responsive-demo', 12);
}

async function recordCodeDemo(browser) {
  console.log('\n💻 Recording code & features demo...');
  const prefix = 'code';
  clearFrames(prefix);

  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });
  let frameNum = 0;

  await page.goto(BASE_URL, { waitUntil: 'networkidle0' });
  await sleep(500);

  // Capture home page
  for (let i = 0; i < 10; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(50);
  }

  // Get sidebar links
  const links = await page.$$eval('nav a, .sidebar a, aside a, .nav-link', els =>
    els.map(el => ({ href: el.href, text: el.textContent?.trim() }))
      .filter(l => l.href && !l.href.includes('#'))
      .slice(0, 5)
  );

  for (const link of links) {
    try {
      console.log(`  📄 Navigating to: ${link.text || link.href}`);
      await page.goto(link.href, { waitUntil: 'networkidle0' });
      await sleep(300);

      // Capture page load
      for (let i = 0; i < 8; i++) {
        await captureFrame(page, prefix, frameNum++);
        await sleep(50);
      }

      // Scroll to show content
      const scrollHeight = await page.evaluate(() => document.body.scrollHeight);
      const viewportHeight = 1080;
      const scrollSteps = Math.min(4, Math.ceil(scrollHeight / viewportHeight));

      for (let step = 0; step < scrollSteps; step++) {
        await page.evaluate((y) => window.scrollTo({ top: y, behavior: 'instant' }),
          (scrollHeight / scrollSteps) * step);
        await sleep(200);

        // Capture scroll position
        for (let i = 0; i < 6; i++) {
          await captureFrame(page, prefix, frameNum++);
          await sleep(50);
        }
      }

      // Scroll back to top
      await page.evaluate(() => window.scrollTo({ top: 0, behavior: 'instant' }));
      await sleep(200);

    } catch (e) {
      console.log(`  ⚠️ Skipped: ${e.message}`);
    }
  }

  // Hold final frame
  for (let i = 0; i < 10; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(50);
  }

  await page.close();
  return framesToWebP(prefix, 'code-demo', 12);
}

async function recordSearchDemo(browser) {
  console.log('\n🔍 Recording search demo...');
  const prefix = 'search';
  clearFrames(prefix);

  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });
  let frameNum = 0;

  await page.goto(BASE_URL, { waitUntil: 'networkidle0' });
  await sleep(500);

  // Initial state
  for (let i = 0; i < 10; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(50);
  }

  // Try to open search with keyboard shortcut (Cmd+K or Ctrl+K)
  console.log('  ⌨️ Opening search with keyboard shortcut...');
  await page.keyboard.down('Meta');
  await page.keyboard.press('KeyK');
  await page.keyboard.up('Meta');
  await sleep(500);

  // Capture search opening
  for (let i = 0; i < 8; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(50);
  }

  // Find search input
  let searchInput = null;
  const inputSelectors = [
    '.pagefind-ui__search-input',
    'input[type="search"]',
    'input[placeholder*="search" i]',
    '.search-input',
    '#search-input',
    'input:visible'
  ];

  for (const sel of inputSelectors) {
    try {
      const el = await page.$(sel);
      if (el) {
        const visible = await el.isIntersectingViewport();
        if (visible) {
          searchInput = el;
          break;
        }
      }
    } catch (e) {}
  }

  if (searchInput) {
    // Type search queries
    const queries = ['markdown', 'syntax', 'config'];

    for (const query of queries) {
      console.log(`  🔤 Typing: "${query}"`);

      // Clear and type character by character
      await searchInput.click({ clickCount: 3 });
      await page.keyboard.press('Backspace');

      for (let i = 0; i < 5; i++) {
        await captureFrame(page, prefix, frameNum++);
        await sleep(30);
      }

      for (const char of query) {
        await searchInput.type(char, { delay: 80 });
        // Capture each keystroke
        for (let i = 0; i < 3; i++) {
          await captureFrame(page, prefix, frameNum++);
          await sleep(30);
        }
      }

      // Wait for results
      await sleep(600);
      for (let i = 0; i < 15; i++) {
        await captureFrame(page, prefix, frameNum++);
        await sleep(50);
      }
    }
  } else {
    console.log('  ⚠️ Search input not found, capturing search button click...');

    // Try clicking search button
    const buttonSelectors = [
      'button[aria-label*="search" i]',
      '.search-button',
      '#search-button',
      '[data-search-trigger]'
    ];

    for (const sel of buttonSelectors) {
      try {
        const btn = await page.$(sel);
        if (btn) {
          await btn.click();
          await sleep(500);
          for (let i = 0; i < 15; i++) {
            await captureFrame(page, prefix, frameNum++);
            await sleep(50);
          }
          break;
        }
      } catch (e) {}
    }
  }

  // Close search
  await page.keyboard.press('Escape');
  await sleep(300);

  // Final frames
  for (let i = 0; i < 10; i++) {
    await captureFrame(page, prefix, frameNum++);
    await sleep(50);
  }

  await page.close();
  return framesToWebP(prefix, 'search-demo', 12);
}

async function main() {
  console.log('🎬 Starting demo recording session...');
  console.log(`Base URL: ${BASE_URL}`);
  console.log(`Output directory: ${GIF_DIR}`);

  // Check if server is running
  try {
    const response = await fetch(BASE_URL);
    if (!response.ok) throw new Error(`Server returned ${response.status}`);
    console.log('✅ Server is running');
  } catch (error) {
    console.error(`\n❌ Server not running at ${BASE_URL}`);
    console.error('Please start the server first:');
    console.error('  cargo run -- -i test_book_mdbook/src -o dist --serve');
    process.exit(1);
  }

  // Check ffmpeg
  try {
    execSync('which ffmpeg', { stdio: 'pipe' });
  } catch {
    console.error('\n❌ ffmpeg not found. Please install it:');
    console.error('  brew install ffmpeg');
    process.exit(1);
  }

  const browser = await puppeteer.launch({
    headless: false,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  try {
    await recordResponsiveDemo(browser);
    await recordCodeDemo(browser);
    await recordSearchDemo(browser);
  } finally {
    await browser.close();
  }

  console.log('\n✅ Demo recording complete!');
  console.log('\nGenerated files in gif/:');
  readdirSync(GIF_DIR)
    .filter(f => f.endsWith('.webp'))
    .forEach(f => console.log(`  - ${f}`));
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
