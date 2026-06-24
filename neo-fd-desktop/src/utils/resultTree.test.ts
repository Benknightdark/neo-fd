import { describe, expect, it } from 'vitest';
import type { ScanResultItem } from '../stores/scan';
import { buildResultTreeRows } from './resultTree';

function result(overrides: Partial<ScanResultItem>): ScanResultItem {
  return {
    id: 1,
    rowKind: 'result',
    path: '/tmp/a.txt',
    line_num: 1,
    pattern_name: '測試',
    matched_text: 'secret',
    ...overrides,
  };
}

describe('buildResultTreeRows', () => {
  it('為同一檔案的多筆匹配建立穩定匹配節點', () => {
    const rows = buildResultTreeRows(
      [
        result({ id: 1, line_num: 10, matched_text: 'a' }),
        result({ id: 2, line_num: 2, matched_text: 'b' }),
      ],
      new Set(),
    );

    const matchRows = rows.filter((row) => row.type === 'match');

    expect(matchRows.map((row) => row.id)).toEqual(['match:2', 'match:1']);
    expect(matchRows.map((row) => row.scanResult?.id)).toEqual([2, 1]);
  });

  it('折疊檔案節點時不輸出子匹配節點', () => {
    const rows = buildResultTreeRows(
      [result({ id: 1 }), result({ id: 2, line_num: 2 })],
      new Set(['path:/tmp/a.txt']),
    );

    expect(rows.some((row) => row.id === 'path:/tmp/a.txt')).toBe(true);
    expect(rows.some((row) => row.type === 'match')).toBe(false);
  });
});
