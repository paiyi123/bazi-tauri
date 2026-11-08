#!/usr/bin/env node
// Upload the pages built by build.py to Confluence (Server / Data Center REST API).
// No dependencies; needs Node 18+.
//
//   node tools/confluence-upload/upload.mjs [--dry-run] [--only <title substring>]
//
// Settings (env vars, defaults in brackets):
//   CONFLUENCE_URL    [http://192.168.1.110:8090]
//   CONFLUENCE_SPACE  [Bazi]
//   CONFLUENCE_ROOT   [159318050]   page id of 「八字命理」
//   CONFLUENCE_TOKEN_FILE [~/.confluence-token]  file holding a Personal Access Token,
//                     or "username:password" for basic auth
//   BUNDLE_DIR        [tools/confluence-upload/out]
// Existing pages (same title in the space) are updated in place; new ones are created.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const BASE = (process.env.CONFLUENCE_URL || 'http://192.168.1.110:8090').replace(/\/$/, '');
const SPACE = process.env.CONFLUENCE_SPACE || 'Bazi';
const ROOT = process.env.CONFLUENCE_ROOT || '159318050';
const BUNDLE = process.env.BUNDLE_DIR || path.join(here, 'out');
const tokenFile = process.env.CONFLUENCE_TOKEN_FILE || path.join(os.homedir(), '.confluence-token');
const args = process.argv.slice(2);
const DRY = args.includes('--dry-run');
const onlyIdx = args.indexOf('--only');
const ONLY = onlyIdx >= 0 ? args[onlyIdx + 1] : null;

if (!fs.existsSync(tokenFile)) {
  console.error(`找不到權杖檔 ${tokenFile}。請把 Confluence 個人存取權杖存進這個檔案。`);
  process.exit(1);
}
const secret = fs.readFileSync(tokenFile, 'utf8').trim();
const AUTH = secret.includes(':')
  ? 'Basic ' + Buffer.from(secret).toString('base64')
  : 'Bearer ' + secret;

async function api(method, url, body, extraHeaders = {}) {
  const headers = { Authorization: AUTH, Accept: 'application/json', ...extraHeaders };
  let payload = body;
  if (body && !(body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
    payload = JSON.stringify(body);
  }
  for (let attempt = 1; ; attempt++) {
    const res = await fetch(BASE + url, { method, headers, body: payload });
    const text = await res.text();
    if (res.ok) return text ? JSON.parse(text) : null;
    if ((res.status === 429 || res.status >= 500) && attempt < 4) {
      await new Promise(r => setTimeout(r, 2000 * attempt));
      continue;
    }
    throw new Error(`${method} ${url} → ${res.status}: ${text.slice(0, 500)}`);
  }
}

async function findPage(title, aliases = []) {
  for (const t of [title, ...aliases]) {
    const q = `/rest/api/content?type=page&spaceKey=${encodeURIComponent(SPACE)}&title=${encodeURIComponent(t)}&expand=version`;
    const r = await api('GET', q);
    if (r.results[0]) return r.results[0];
  }
  return null;
}

async function addLabels(pageId, labels) {
  if (!labels?.length) return;
  await api('POST', `/rest/api/content/${pageId}/label`, labels.map(name => ({ prefix: 'global', name })));
}

// The space home (root page) keeps whatever was on it before, folded into an expand at the bottom.
async function updateHome(p) {
  const cur = await api('GET', `/rest/api/content/${ROOT}?expand=body.storage,version`);
  let value = fs.readFileSync(path.join(BUNDLE, p.file), 'utf8');
  const old = cur.body.storage.value || '';
  if (old.trim() && !old.includes('bazi-home')) {
    value += '<ac:structured-macro ac:name="expand"><ac:parameter ac:name="title">原本的頁面內容</ac:parameter>'
      + `<ac:rich-text-body>${old}</ac:rich-text-body></ac:structured-macro>`;
  }
  if (DRY) { console.log(`首頁 ${cur.title}：會更新（原有內容${old.trim() && !old.includes('bazi-home') ? '保留在最下方' : '是之前產生的，直接取代'}）`); return; }
  await api('PUT', `/rest/api/content/${ROOT}`, {
    id: ROOT, type: 'page', title: cur.title,
    version: { number: cur.version.number + 1, message: '首頁導覽' },
    body: { storage: { value, representation: 'storage' } },
  });
  await addLabels(ROOT, p.labels);
  console.log(`首頁 ${cur.title}：已更新`);
}

async function upsertAttachment(pageId, file) {
  const name = path.basename(file);
  const existing = await api('GET', `/rest/api/content/${pageId}/child/attachment?filename=${encodeURIComponent(name)}`);
  const form = new FormData();
  form.append('file', new Blob([fs.readFileSync(file)]), name);
  form.append('minorEdit', 'true');
  const url = existing.results.length
    ? `/rest/api/content/${pageId}/child/attachment/${existing.results[0].id}/data`
    : `/rest/api/content/${pageId}/child/attachment`;
  await api('POST', url, form, { 'X-Atlassian-Token': 'no-check' });
}

const manifest = JSON.parse(fs.readFileSync(path.join(BUNDLE, 'manifest.json'), 'utf8'));
const root = await api('GET', `/rest/api/content/${ROOT}?expand=space`);
console.log(`根頁面：${root.title}（空間 ${root.space.key}）`);
if (root.space.key !== SPACE) throw new Error(`根頁面不在空間 ${SPACE}`);

const ids = { ROOT };
let created = 0, updated = 0, skipped = 0;
for (const [i, p] of manifest.entries()) {
  if (p.key === 'HOME') {
    if (!ONLY || p.title.includes(ONLY)) await updateHome(p);
    continue;
  }
  const parentId = ids[p.parent];
  const tag = `[${i + 1}/${manifest.length}] ${p.title}`;
  const existing = await findPage(p.title, p.aliases);
  if (ONLY && !p.title.includes(ONLY)) {
    if (existing) ids[p.key] = existing.id;
    else if (!DRY) { console.log(`${tag}：略過（--only），且頁面尚未存在，其子頁也會略過`); }
    skipped++;
    continue;
  }
  if (!parentId) { console.log(`${tag}：上層頁面不存在，略過`); skipped++; continue; }
  const value = fs.readFileSync(path.join(BUNDLE, p.file), 'utf8');
  const body = { storage: { value, representation: 'storage' } };
  if (DRY) {
    const rename = existing && existing.title !== p.title ? `，改名自「${existing.title}」` : '';
    console.log(`${tag}：${existing ? '會更新' : '會建立'}（${Math.round(p.bytes / 1024)} KB${rename}）`);
    ids[p.key] = existing ? existing.id : `new:${p.key}`;
    continue;
  }
  let page;
  if (existing) {
    page = await api('PUT', `/rest/api/content/${existing.id}`, {
      id: existing.id, type: 'page', title: p.title,
      ancestors: [{ id: parentId }],
      version: { number: existing.version.number + 1, minorEdit: true, message: '由 bazi 筆記同步' },
      body,
    });
    updated++;
  } else {
    page = await api('POST', '/rest/api/content', {
      type: 'page', title: p.title, space: { key: SPACE },
      ancestors: [{ id: parentId }], body,
    });
    created++;
  }
  ids[p.key] = page.id;
  await addLabels(page.id, p.labels);
  for (const a of p.attachments) await upsertAttachment(page.id, path.join(BUNDLE, 'attachments', a));
  console.log(`${tag}：${existing ? '已更新' : '已建立'} ${BASE}/pages/viewpage.action?pageId=${page.id}`);
}
console.log(DRY ? '（試跑，沒有寫入）' : `完成：建立 ${created}、更新 ${updated}、略過 ${skipped}`);
