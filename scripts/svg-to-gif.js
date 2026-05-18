#!/usr/bin/env node
/**
 * Convert asciinema SVG to optimized GIF.
 * Renders key frames only and creates a compact GIF.
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const CAST_PATH = '/home/z/my-project/codescope/demo.cast';
const SVG_PATH = '/home/z/my-project/codescope/demo.svg';
const OUTPUT_DIR = '/home/z/my-project/codescope/demo-frames';
const OUTPUT_GIF = '/home/z/my-project/codescope/assets/demo.gif';
const TARGET_FPS = 6;
const GIF_WIDTH = 780;

async function main() {
    // Read cast file for frame timing
    const castLines = fs.readFileSync(CAST_PATH, 'utf-8').trim().split('\n');
    const frames = castLines.slice(1).map(l => JSON.parse(l));
    const totalDurationMs = frames[frames.length - 1][0] * 1000;
    
    // Clean frames dir
    if (fs.existsSync(OUTPUT_DIR)) fs.rmSync(OUTPUT_DIR, { recursive: true });
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    
    // Read SVG
    const svgContent = fs.readFileSync(SVG_PATH, 'utf-8');
    
    const htmlPath = '/tmp/cs-demo-render.html';
    fs.writeFileSync(htmlPath, `<!DOCTYPE html>
<html><head><style>
body { margin:0; padding:0; background:#1e1e2e; display:flex; justify-content:center; align-items:center; min-height:100vh; overflow:hidden; }
svg { max-width:${GIF_WIDTH}px; width:100%; height:auto; }
</style></head><body>${svgContent}</body></html>`);
    
    const browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({ viewport: { width: GIF_WIDTH + 20, height: 520 } });
    await page.goto('file://' + htmlPath, { waitUntil: 'networkidle' });
    await page.waitForTimeout(500);
    
    // Render at target FPS
    const numFrames = Math.ceil(totalDurationMs / 1000 * TARGET_FPS);
    console.log(`Duration: ${(totalDurationMs/1000).toFixed(1)}s, Frames: ${numFrames} @ ${TARGET_FPS}fps`);
    
    for (let i = 0; i < numFrames; i++) {
        const outPath = path.join(OUTPUT_DIR, `f${String(i).padStart(4, '0')}.png`);
        await page.screenshot({ path: outPath, type: 'png' });
        if (i % 5 === 0) process.stdout.write(`\r  ${i}/${numFrames}`);
    }
    console.log(`\r  ${numFrames}/${numFrames} screenshots done`);
    await browser.close();
    
    // Build GIF with ffmpeg (2-pass for quality)
    const palette = path.join(OUTPUT_DIR, 'pal.png');
    
    // Pass 1: generate palette
    execSync(
        `ffmpeg -y -framerate ${TARGET_FPS} -i ${OUTPUT_DIR}/f%04d.png ` +
        `-vf "scale=${GIF_WIDTH}:-1:flags=lanczos,palettegen=max_colors=128:stats_mode=diff" ${palette}`,
        { stdio: 'pipe' }
    );
    
    // Pass 2: create GIF
    execSync(
        `ffmpeg -y -framerate ${TARGET_FPS} -i ${OUTPUT_DIR}/f%04d.png -i ${palette} ` +
        `-lavfi "scale=${GIF_WIDTH}:-1:flags=lanczos [x]; [x][1:v] paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle" ${OUTPUT_GIF}`,
        { stdio: 'pipe' }
    );
    
    const size = fs.statSync(OUTPUT_GIF).size;
    console.log(`GIF: ${OUTPUT_GIF} (${(size/1024).toFixed(0)} KB, ${numFrames} frames, ${TARGET_FPS}fps)`);
}

main().catch(e => { console.error(e.message); process.exit(1); });
