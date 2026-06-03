#!/usr/bin/env node
/**
 * 生成 C01–C12 并排走查 HTML（BCIP vs Codex 参考）。
 * 用法：node scripts/generate-walkthrough-review.mjs [--open-hint]
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const SHOT_DIR = path.join(ROOT, 'docs/walkthrough-screenshots');
const REF_DIR = path.join(SHOT_DIR, 'codex-ref');
const CHECKLIST_PATH = path.join(ROOT, 'docs/walkthrough-checklist.json');
const OUT_HTML = path.join(SHOT_DIR, 'review.html');

const checklist = JSON.parse(fs.readFileSync(CHECKLIST_PATH, 'utf8'));
const generatedAt = new Date().toISOString();

function rel(fromDir, target) {
  return path.relative(fromDir, target).split(path.sep).join('/');
}

function imgOrPlaceholder(dir, id, label) {
  const file = path.join(dir, `${id}.png`);
  if (fs.existsSync(file)) {
    const href = rel(SHOT_DIR, file);
    return `<img src="${href}" alt="${label} ${id}" loading="lazy" />`;
  }
  return `<div class="placeholder"><span>${label} ${id}</span><small>未提供 ${path.basename(dir)}/${id}.png</small></div>`;
}

function criteriaHtml(item) {
  return item.criteria
    .map(
      (c) => `
        <label class="criterion">
          <input type="checkbox" data-criterion-id="${c.id}" />
          <span>${escapeHtml(c.text)}</span>
        </label>`,
    )
    .join('');
}

function escapeHtml(s) {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

const cards = checklist.items
  .map(
    (item) => `
    <section class="card" id="${item.id}" data-item-id="${item.id}">
      <header class="card-head">
        <h2>${item.id} · ${escapeHtml(item.title)}</h2>
        <p class="meta"><code>${escapeHtml(item.component)}</code></p>
        <p class="hint">${escapeHtml(item.captureHint)}</p>
      </header>
      <div class="compare">
        <figure>
          <figcaption>BCIP 当前</figcaption>
          ${imgOrPlaceholder(SHOT_DIR, item.id, 'BCIP')}
        </figure>
        <figure>
          <figcaption>Codex 参考</figcaption>
          ${imgOrPlaceholder(REF_DIR, item.id, 'Codex')}
        </figure>
      </div>
      <div class="criteria">${criteriaHtml(item)}</div>
      <label class="verdict">
        <span>设计结论</span>
        <select data-verdict-id="${item.id}">
          <option value="">— 待评审 —</option>
          <option value="pass">✓ 通过</option>
          <option value="minor">△ 轻微偏差（可发版）</option>
          <option value="fail">✗ 需修复</option>
          <option value="na">— 不适用</option>
        </select>
      </label>
      <textarea data-notes-id="${item.id}" rows="2" placeholder="评审备注（间距、色值、动画等）…"></textarea>
    </section>`,
  )
  .join('\n');

const bcipOnly = (checklist.bcipOnly ?? [])
  .map(
    (p) =>
      `<li><strong>${p.id}</strong> ${escapeHtml(p.title)} — <code>${escapeHtml(p.component)}</code> — ${escapeHtml(p.note)}</li>`,
  )
  .join('\n');

const html = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>BCIP 桌面端设计走查 · C01–C12</title>
  <style>
    :root {
      --bg: #1a1816;
      --surface: #252220;
      --border: rgba(255,255,255,0.08);
      --text: #f5f2ee;
      --muted: #a39e98;
      --accent: #4a7c6f;
      --pass: #3d8b5e;
      --warn: #c49a3a;
      --fail: #c44;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "Inter", system-ui, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
    }
    .toolbar {
      position: sticky;
      top: 0;
      z-index: 10;
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      align-items: center;
      padding: 12px 20px;
      background: rgba(26,24,22,0.92);
      backdrop-filter: blur(8px);
      border-bottom: 1px solid var(--border);
    }
    .toolbar h1 { font-size: 15px; margin: 0; font-weight: 600; }
    .toolbar .sub { font-size: 12px; color: var(--muted); }
    .toolbar button {
      padding: 6px 12px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--text);
      cursor: pointer;
      font-size: 12px;
    }
    .toolbar button.primary { background: var(--accent); border-color: transparent; }
    .progress { font-size: 12px; color: var(--muted); margin-left: auto; }
    main { max-width: 1200px; margin: 0 auto; padding: 20px; }
    .intro {
      font-size: 13px;
      color: var(--muted);
      margin-bottom: 24px;
      padding: 12px 16px;
      background: var(--surface);
      border-radius: 8px;
      border: 1px solid var(--border);
    }
    .card {
      margin-bottom: 32px;
      padding: 16px;
      background: var(--surface);
      border-radius: 10px;
      border: 1px solid var(--border);
    }
    .card-head h2 { margin: 0 0 4px; font-size: 16px; }
    .meta, .hint { margin: 0; font-size: 12px; color: var(--muted); }
    .compare {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 12px;
      margin: 16px 0;
    }
    @media (max-width: 800px) { .compare { grid-template-columns: 1fr; } }
    figure { margin: 0; }
    figcaption {
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--muted);
      margin-bottom: 6px;
    }
    figure img {
      width: 100%;
      height: auto;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: #111;
    }
    .placeholder {
      min-height: 120px;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      border: 1px dashed var(--border);
      border-radius: 6px;
      color: var(--muted);
      font-size: 12px;
      gap: 4px;
    }
    .criteria { display: flex; flex-direction: column; gap: 6px; margin-bottom: 12px; }
    .criterion {
      display: flex;
      gap: 8px;
      font-size: 13px;
      cursor: pointer;
    }
    .criterion input { margin-top: 3px; accent-color: var(--accent); }
    .verdict {
      display: flex;
      align-items: center;
      gap: 10px;
      font-size: 13px;
      margin-bottom: 8px;
    }
    .verdict select {
      flex: 1;
      max-width: 220px;
      padding: 4px 8px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: var(--bg);
      color: var(--text);
    }
    textarea {
      width: 100%;
      padding: 8px 10px;
      border-radius: 6px;
      border: 1px solid var(--border);
      background: var(--bg);
      color: var(--text);
      font-size: 12px;
      resize: vertical;
    }
    .bcip-only { font-size: 13px; color: var(--muted); }
    .bcip-only ul { padding-left: 20px; }
    .export-preview {
      display: none;
      white-space: pre-wrap;
      font-family: ui-monospace, monospace;
      font-size: 11px;
      padding: 12px;
      background: var(--bg);
      border-radius: 6px;
      margin-top: 12px;
      max-height: 320px;
      overflow: auto;
    }
  </style>
</head>
<body>
  <div class="toolbar">
    <div>
      <h1>BCIP × Codex 像素走查</h1>
      <div class="sub">生成于 ${generatedAt} · 视口 ${checklist.viewport.width}×${checklist.viewport.height} · 规范 ${escapeHtml(checklist.specRef)}</div>
    </div>
    <button type="button" id="btn-save">保存进度（本地）</button>
    <button type="button" class="primary" id="btn-export">导出 Markdown 签收单</button>
    <span class="progress" id="progress">—</span>
  </div>
  <main>
    <div class="intro">
      <strong>使用说明：</strong>左侧为 <code>npm run walkthrough:capture</code> 产出的 BCIP 截图；
      右侧为可选 Codex 参考（放入 <code>codex-ref/C01.png</code> … <code>C12.png</code>）。
      勾选验收项、选择结论并填写备注；点击「导出 Markdown」生成 PR 签收附件。进度保存在浏览器 localStorage（键 <code>bcip-walkthrough-review</code>）。
    </div>
    ${cards}
    <section class="card bcip-only">
      <h2>BCIP-only（不要求与 Codex 一致）</h2>
      <ul>${bcipOnly}</ul>
    </section>
    <pre class="export-preview" id="export-preview"></pre>
  </main>
  <script>
    const STORAGE_KEY = 'bcip-walkthrough-review';
    const ITEM_IDS = ${JSON.stringify(checklist.items.map((i) => i.id))};

    function loadState() {
      try {
        return JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}');
      } catch {
        return {};
      }
    }

    function saveState(state) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    }

    function collectState() {
      const state = loadState();
      state.criteria = {};
      document.querySelectorAll('[data-criterion-id]').forEach((el) => {
        state.criteria[el.dataset.criterionId] = el.checked;
      });
      state.verdicts = {};
      document.querySelectorAll('[data-verdict-id]').forEach((el) => {
        state.verdicts[el.dataset.verdictId] = el.value;
      });
      state.notes = {};
      document.querySelectorAll('[data-notes-id]').forEach((el) => {
        state.notes[el.dataset.notesId] = el.value;
      });
      state.updatedAt = new Date().toISOString();
      return state;
    }

    function applyState(state) {
      if (!state) return;
      document.querySelectorAll('[data-criterion-id]').forEach((el) => {
        if (state.criteria?.[el.dataset.criterionId] != null) {
          el.checked = state.criteria[el.dataset.criterionId];
        }
      });
      document.querySelectorAll('[data-verdict-id]').forEach((el) => {
        if (state.verdicts?.[el.dataset.verdictId] != null) {
          el.value = state.verdicts[el.dataset.verdictId];
        }
      });
      document.querySelectorAll('[data-notes-id]').forEach((el) => {
        if (state.notes?.[el.dataset.notesId] != null) {
          el.value = state.notes[el.dataset.notesId];
        }
      });
    }

    function updateProgress() {
      const state = collectState();
      const done = ITEM_IDS.filter((id) => state.verdicts[id]).length;
      const pass = ITEM_IDS.filter((id) => state.verdicts[id] === 'pass').length;
      const fail = ITEM_IDS.filter((id) => state.verdicts[id] === 'fail').length;
      document.getElementById('progress').textContent =
        '已评审 ' + done + '/' + ITEM_IDS.length + ' · 通过 ' + pass + ' · 待修 ' + fail;
    }

    function exportMarkdown() {
      const state = collectState();
      const lines = [
        '# BCIP 桌面端设计走查签收单',
        '',
        '- 生成时间: ' + state.updatedAt,
        '- 规范: ${escapeHtml(checklist.specRef)}',
        '- 视口: ${checklist.viewport.width}×${checklist.viewport.height}',
        '',
        '## 汇总',
        '',
      ];
      const verdictLabels = { pass: '通过', minor: '轻微偏差', fail: '需修复', na: '不适用' };
      let pass = 0, minor = 0, fail = 0, pending = 0;
      ITEM_IDS.forEach((id) => {
        const v = state.verdicts[id];
        if (v === 'pass') pass++;
        else if (v === 'minor') minor++;
        else if (v === 'fail') fail++;
        else pending++;
      });
      lines.push('| 通过 | 轻微偏差 | 需修复 | 待评审 |');
      lines.push('|------|----------|--------|--------|');
      lines.push('| ' + pass + ' | ' + minor + ' | ' + fail + ' | ' + pending + ' |');
      lines.push('', '## 逐项', '');
      ITEM_IDS.forEach((id) => {
        const v = state.verdicts[id] || '待评审';
        const label = verdictLabels[v] || v;
        lines.push('### ' + id);
        lines.push('- **结论**: ' + label);
        if (state.notes[id]) lines.push('- **备注**: ' + state.notes[id]);
        lines.push('');
      });
      lines.push('## 签收', '', '| 角色 | 姓名 | 日期 |', '|------|------|------|', '| 设计 | | |', '| 前端 | | |', '| QA | | |', '');
      const md = lines.join('\\n');
      const preview = document.getElementById('export-preview');
      preview.style.display = 'block';
      preview.textContent = md;
      navigator.clipboard?.writeText(md);
      alert('Markdown 已复制到剪贴板（若浏览器允许），并显示在页面底部。');
    }

    applyState(loadState());
    updateProgress();

    document.querySelectorAll('input, select, textarea').forEach((el) => {
      el.addEventListener('change', () => {
        saveState(collectState());
        updateProgress();
      });
      el.addEventListener('input', () => {
        saveState(collectState());
        updateProgress();
      });
    });

    document.getElementById('btn-save').addEventListener('click', () => {
      saveState(collectState());
      updateProgress();
      alert('已保存到 localStorage');
    });
    document.getElementById('btn-export').addEventListener('click', exportMarkdown);
  </script>
</body>
</html>`;

fs.mkdirSync(SHOT_DIR, { recursive: true });
fs.writeFileSync(OUT_HTML, html, 'utf8');

console.log(`已生成: ${OUT_HTML}`);
console.log('在浏览器中打开 review.html 进行并排评审。');

if (process.argv.includes('--open-hint')) {
  console.log(`\n  open "${OUT_HTML}"`);
}
