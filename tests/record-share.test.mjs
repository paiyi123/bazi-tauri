import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const cacheDir = mkdtempSync(join(tmpdir(), 'bazi-record-share-'));
const server = await createServer({ root: process.cwd(), cacheDir, server: { middlewareMode: true }, appType: 'custom', optimizeDeps: { noDiscovery: true, entries: [] } });
after(async () => {
  await server.close();
  rmSync(cacheDir, { recursive: true, force: true });
});
const { encodeRecordShare, decodeRecordShare, importSharedRecord, encodeRecordsShare, decodeRecordsShare, chunkRecordsForShare, importSharedRecords } = await server.ssrLoadModule('/src/utils/recordShare.ts');

const record = {
  name: '王小明',
  gender: 'FEMALE',
  calendarType: 'LUNAR',
  yearEra: 'ROC',
  year: 59,
  month: 10,
  day: 8,
  hour: 5,
  minute: 0,
  leapMonth: true,
  timeLabel: '卯時 05:00',
  notes: '課堂案例 "A"',
};

test('分享碼可以原樣還原', () => {
  const code = encodeRecordShare(record);
  assert.ok(code.startsWith('BAZI1:'));
  assert.deepEqual(decodeRecordShare(code), record);
  assert.deepEqual(decodeRecordShare(`  前面有字 ${code}\n`), record);
});

test('分享碼很短，適合放進 QR Code', () => {
  assert.ok(new TextEncoder().encode(encodeRecordShare(record)).length < 150);
});

test('錯誤的分享碼給中文錯誤訊息', () => {
  assert.throws(() => decodeRecordShare('hello'), /不是命例分享碼/);
  assert.throws(() => decodeRecordShare('BAZI1:["王"'), /不完整/);
  assert.throws(() => decodeRecordShare('BAZI1:["王","F","S","A",1970,13,6,5,0,0]'), /月份/);
  assert.throws(() => decodeRecordShare('BAZI1:["","F","S","A",1970,11,6,5,0,0]'), /姓名/);
});

test('匯入：新命例加在最前面，編號接續', () => {
  const existing = [{ ...record, name: '甲', id: 7, createdAt: 'x', updatedAt: 'x' }];
  const result = importSharedRecord(existing, record, '2026-10-29T00:00:00.000Z');
  assert.equal(result.kind, 'added');
  assert.equal(result.record.id, 8);
  assert.equal(result.records.length, 2);
  assert.equal(result.records[0].name, '王小明');
});

test('匯入：同名同出生資料不重複新增', () => {
  const existing = [{ ...record, notes: '別的備註', id: 3, createdAt: 'x', updatedAt: 'x' }];
  const result = importSharedRecord(existing, record);
  assert.equal(result.kind, 'duplicate');
  assert.equal(result.record.id, 3);
  assert.equal(result.records, existing);
});

const many = Array.from({ length: 23 }, (_, i) => ({ ...record, name: `案例${i + 1}`, notes: '', day: (i % 28) + 1 }));

test('多筆分享碼可以還原，也能一次貼上好幾張', () => {
  const code = encodeRecordsShare(many.slice(0, 3));
  assert.deepEqual(decodeRecordsShare(code), many.slice(0, 3));
  const chunks = chunkRecordsForShare(many);
  assert.deepEqual(chunks.map((c) => c.records.length), [10, 10, 3]);
  assert.ok(chunks.every((c) => new TextEncoder().encode(c.code).length <= 900));
  assert.deepEqual(decodeRecordsShare(chunks.map((c) => c.code).join('\n')), many);
});

test('備註很長時依位元組上限提早分張', () => {
  const long = many.slice(0, 10).map((r) => ({ ...r, notes: '很長的備註'.repeat(10) }));
  const chunks = chunkRecordsForShare(long);
  assert.ok(chunks.length > 1);
  assert.equal(chunks.flatMap((c) => c.records).length, 10);
});

test('多筆匯入：分別統計新增與已存在', () => {
  const existing = [{ ...many[0], id: 1, createdAt: 'x', updatedAt: 'x' }];
  const result = importSharedRecords(existing, many.slice(0, 3));
  assert.equal(result.added.length, 2);
  assert.equal(result.duplicates.length, 1);
  assert.equal(result.records.length, 3);
  assert.deepEqual(result.added.map((r) => r.id), [2, 3]);
});
