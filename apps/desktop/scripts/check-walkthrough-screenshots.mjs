#!/usr/bin/env node
/**
 * 校验 C01–C12 走查截图是否齐全。
 * 用法：node scripts/check-walkthrough-screenshots.mjs [--strict-ref]
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const SHOT_DIR = path.join(ROOT, 'docs/walkthrough-screenshots');
const REF_DIR = path.join(SHOT_DIR, 'codex-ref');
const IDS = Array.from({ length: 12 }, (_, i) => `C${String(i + 1).padStart(2, '0')}`);

const strictRef = process.argv.includes('--strict-ref');

function checkPng(dir, id) {
  const file = path.join(dir, `${id}.png`);
  if (!fs.existsSync(file)) {
    return { ok: false, file, reason: 'missing' };
  }
  const stat = fs.statSync(file);
  if (stat.size < 500) {
    return { ok: false, file, reason: 'too_small', bytes: stat.size };
  }
  return { ok: true, file, bytes: stat.size };
}

let failed = false;

console.log('BCIP 走查截图检查');
console.log(`目录: ${SHOT_DIR}\n`);

const bcipResults = IDS.map((id) => ({ id, ...checkPng(SHOT_DIR, id) }));
const missingBcip = bcipResults.filter((r) => !r.ok);

if (missingBcip.length === 0) {
  for (const r of bcipResults) {
    console.log(`  ✓ ${r.id}.png (${r.bytes} bytes)`);
  }
} else {
  failed = true;
  for (const r of bcipResults) {
    if (r.ok) {
      console.log(`  ✓ ${r.id}.png`);
    } else {
      console.log(`  ✗ ${r.id}.png — ${r.reason}${r.bytes != null ? ` (${r.bytes} B)` : ''}`);
    }
  }
  console.log('\n缺少 BCIP 截图。请先运行: npm run walkthrough:capture');
}

const refResults = IDS.map((id) => ({ id, ...checkPng(REF_DIR, id) }));
const refPresent = refResults.filter((r) => r.ok).length;

console.log(`\nCodex 参考图: ${refPresent}/${IDS.length}（目录 codex-ref/）`);
if (refPresent > 0 && refPresent < IDS.length) {
  for (const r of refResults) {
    if (!r.ok) {
      console.log(`  ○ 可选缺失 ${r.id}.png`);
    }
  }
}

if (strictRef && refPresent < IDS.length) {
  failed = true;
  console.error('\n--strict-ref: 要求 codex-ref/ 下 C01–C12 全部存在');
}

if (failed) {
  process.exit(1);
}

console.log('\n检查通过。');
