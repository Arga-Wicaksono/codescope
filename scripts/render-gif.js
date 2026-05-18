#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const { chromium } = require('playwright');

const CAST = '/home/z/my-project/codescope/demo.cast';
const DIR = '/home/z/my-project/codescope/demo-frames';
const GIF = '/home/z/my-project/codescope/assets/demo.gif';
const W = 700, H = 460;

const C = {
  '0':'color:#6272a4','30':'color:#21222c','31':'color:#ff5555','32':'color:#50fa7b',
  '33':'color:#f1fa8c','34':'color:#6272a4','35':'color:#ff79c6','36':'color:#8be9fd',
  '37':'color:#f8f8f2','39':'color:#f8f8f2','1':'font-weight:bold','2':'opacity:0.45'
};

function a2h(t) {
  t = t.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/\r/g,'');
  t = t.replace(/\x1b\[0m/g, '</span>');
  t = t.replace(/\x1b\[([0-9;]*)m/g, function(_, codes) {
    var s = '';
    codes.split(';').forEach(function(c) { if (C[c]) s += C[c] + ';'; });
    return s ? '<span style="' + s + '">' : '';
  });
  return t.replace(/\n/g, '<br>');
}

function makeHtml(content) {
  return '<!DOCTYPE html><html><head><style>' +
    '*{margin:0;padding:0;box-sizing:border-box}' +
    'body{background:#282a36;display:flex;justify-content:center;align-items:center;min-height:100vh}' +
    '.term{background:#1e1e2e;border:1px solid #313244;border-radius:10px;padding:14px 18px;width:' + W + 'px;' +
    'font-family:Menlo,"SF Mono","Courier New",monospace;font-size:13px;line-height:16.5px;' +
    'color:#cdd6f4;overflow:hidden;box-shadow:0 6px 24px rgba(0,0,0,0.5)}' +
    '.bar{display:flex;gap:5px;margin-bottom:10px;padding-bottom:8px;border-bottom:1px solid #313244;align-items:center}' +
    '.dot{width:9px;height:9px;border-radius:50%}.r{background:#f38ba8}.y{background:#f9e2af}.g{background:#a6e3a1}' +
    '.t{flex:1;text-align:center;color:#6c7086;font-size:10.5px}' +
    '.c{white-space:pre-wrap;word-break:break-all}' +
    '</style></head><body><div class="term"><div class="bar">' +
    '<div class="dot r"></div><div class="dot y"></div><div class="dot g"></div>' +
    '<div class="t">cs — CodeScope</div></div><div class="c">' + a2h(content) + '</div>' +
    '</div></body></html>';
}

async function main() {
  var lines = fs.readFileSync(CAST, 'utf-8').trim().split('\n');
  var events = [];
  for (var i = 1; i < lines.length; i++) {
    var f = JSON.parse(lines[i]);
    if (f[1] === 'o' && f[2]) events.push(f);
  }

  // Build progressive frames, but SKIP frames where text change is just whitespace
  var frames = [{time: 0, text: ''}];
  var text = '';
  var prevLen = 0;
  for (var j = 0; j < events.length; j++) {
    text += events[j][2];
    // Only add frame if meaningful new content was added (more than just \n)
    var added = events[j][2].replace(/\n/g, '').replace(/\x1b\[[0-9;]*m/g, '');
    if (added.length > 0 || events[j][2].length > 2) {
      frames.push({time: events[j][0], text: text});
    }
  }
  // Remove very similar consecutive frames (same visible content)
  var reduced = [frames[0]];
  for (var k = 1; k < frames.length; k++) {
    var prevVisible = frames[k-1].text.replace(/\x1b\[[0-9;]*m/g, '').replace(/\s+/g,' ').trim();
    var currVisible = frames[k].text.replace(/\x1b\[[0-9;]*m/g, '').replace(/\s+/g,' ').trim();
    if (prevVisible !== currVisible) {
      reduced.push(frames[k]);
    }
  }

  frames = reduced;
  console.log('Events: ' + events.length + ' -> Key frames: ' + frames.length);
  console.log('Duration: ' + frames[frames.length-1].time.toFixed(1) + 's');

  if (fs.existsSync(DIR)) fs.rmSync(DIR, {recursive:true});
  fs.mkdirSync(DIR, {recursive:true});

  var browser = await chromium.launch({headless:true});
  var page = await browser.newPage({viewport:{width:W+30, height:H+30}});

  for (var m = 0; m < frames.length; m++) {
    var p = path.join(DIR, 'f' + String(m).padStart(4,'0') + '.png');
    var hp = '/tmp/cs-rf-' + m + '.html';
    fs.writeFileSync(hp, makeHtml(frames[m].text));
    await page.goto('file://' + hp, {waitUntil:'load'});
    await page.waitForTimeout(40);
    await page.screenshot({path:p, type:'png', clip:{x:0,y:0,width:W+20,height:H+20}});
  }
  console.log('Screenshots: ' + frames.length);
  await browser.close();

  // Build concat list with per-frame durations
  var durLines = [];
  for (var n = 0; n < frames.length; n++) {
    durLines.push('file f' + String(n).padStart(4,'0') + '.png');
    var dur = n === 0 ? Math.max(frames[0].time, 0.3) : frames[n].time - frames[n-1].time;
    durLines.push('duration ' + Math.max(dur, 0.25).toFixed(3));
  }
  durLines.push('file f' + String(frames.length-1).padStart(4,'0') + '.png');
  fs.writeFileSync(path.join(DIR, 'list.txt'), durLines.join('\n'));

  console.log('Compiling GIF...');
  execSync(
    'ffmpeg -y -f concat -safe 0 -i ' + DIR + '/list.txt ' +
    '-vf "scale=' + W + ':-1:flags=lanczos,palettegen=max_colors=64:stats_mode=diff" ' +
    DIR + '/palette.png',
    {stdio:'pipe'}
  );
  execSync(
    'ffmpeg -y -f concat -safe 0 -i ' + DIR + '/list.txt -i ' + DIR + '/palette.png ' +
    '-lavfi "scale=' + W + ':-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5" ' +
    GIF,
    {stdio:'pipe'}
  );

  var size = fs.statSync(GIF).size;
  console.log('\nGIF: ' + GIF + ' (' + (size/1024).toFixed(0) + ' KB, ' + frames.length + ' frames)');
}

main().catch(function(e) { console.error(e.message); process.exit(1); });
