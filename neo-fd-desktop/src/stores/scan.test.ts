import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useScanStore } from './scan';

const eventListeners = vi.hoisted(
  () => new Map<string, (event: { payload: unknown }) => void>(),
);

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(
    (eventName: string, handler: (event: { payload: unknown }) => void) => {
      eventListeners.set(eventName, handler);
      return Promise.resolve(() => {
        eventListeners.delete(eventName);
      });
    },
  ),
}));

describe('useScanStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    eventListeners.clear();
    vi.mocked(invoke).mockClear();
    vi.mocked(invoke).mockResolvedValue(undefined);
    vi.mocked(listen).mockClear();
  });

  it('將空白最大匹配筆數轉為 null', () => {
    const store = useScanStore();

    expect(store.normalizeMaxResults('')).toBeNull();
    expect(store.normalizeMaxResults('   ')).toBeNull();
  });

  it('接受大於 0 的整數最大匹配筆數', () => {
    const store = useScanStore();

    expect(store.normalizeMaxResults('1')).toBe(1);
    expect(store.normalizeMaxResults(' 25 ')).toBe(25);
  });

  it('拒絕非正整數最大匹配筆數', () => {
    const store = useScanStore();

    expect(() => store.normalizeMaxResults('0')).toThrow(
      '最大匹配筆數必須是大於 0 的整數',
    );
    expect(() => store.normalizeMaxResults('-1')).toThrow(
      '最大匹配筆數必須是大於 0 的整數',
    );
    expect(() => store.normalizeMaxResults('1.5')).toThrow(
      '最大匹配筆數必須是大於 0 的整數',
    );
    expect(() => store.normalizeMaxResults('abc')).toThrow(
      '最大匹配筆數必須是大於 0 的整數',
    );
  });

  it('啟動掃描時將空白最大匹配筆數傳為 null', async () => {
    const store = useScanStore();

    await store.startScan();

    expect(invoke).toHaveBeenCalledWith(
      'scan_directory',
      expect.objectContaining({
        maxResults: null,
      }),
    );
  });

  it('啟動掃描時記錄開始時間並清空結束時間', async () => {
    const store = useScanStore();
    store.scanEndTime = new Date('2026-06-24T07:00:00.000Z');

    await store.startScan();

    expect(store.scanStartTime).toBeInstanceOf(Date);
    expect(store.scanEndTime).toBeNull();
  });

  it('取消掃描成功時記錄結束時間', async () => {
    const store = useScanStore();
    store.isScanning = true;

    await store.cancelScan();

    expect(invoke).toHaveBeenCalledWith('cancel_scan', {});
    expect(store.scanEndTime).toBeInstanceOf(Date);
  });

  it('掃描完成事件會補上結束時間', async () => {
    const store = useScanStore();
    await store.init();

    eventListeners.get('scan-finished')?.({ payload: undefined });

    expect(store.scanEndTime).toBeInstanceOf(Date);
  });

  it('批次加入掃描結果時建立穩定 ID 與檔案路徑索引', async () => {
    const store = useScanStore();
    await store.init();

    eventListeners.get('scan-result-batch')?.({
      payload: [
        {
          path: '/tmp/a.txt',
          line_num: 1,
          pattern_name: '測試',
          matched_text: 'secret',
        },
        {
          path: '/tmp/a.txt',
          line_num: 2,
          pattern_name: '測試',
          matched_text: 'secret2',
        },
      ],
    });
    eventListeners.get('scan-finished')?.({ payload: undefined });

    expect(store.results).toHaveLength(2);
    expect(store.results.map((result) => result.id)).toEqual([1, 2]);
    expect(store.results.every((result) => result.rowKind === 'result')).toBe(
      true,
    );
    expect(
      store.getResultsByPath('/tmp/a.txt').map((result) => result.id),
    ).toEqual([1, 2]);
  });

  it('依檔案路徑刪除結果時同步更新結果列表與索引', async () => {
    const store = useScanStore();
    await store.init();

    eventListeners.get('scan-result-batch')?.({
      payload: [
        {
          path: '/tmp/a.txt',
          line_num: 1,
          pattern_name: '測試',
          matched_text: 'secret',
        },
        {
          path: '/tmp/b.txt',
          line_num: 1,
          pattern_name: '測試',
          matched_text: 'secret',
        },
      ],
    });
    eventListeners.get('scan-finished')?.({ payload: undefined });

    store.removeResultsByPath('/tmp/a.txt');

    expect(store.results.map((result) => result.path)).toEqual(['/tmp/b.txt']);
    expect(store.getResultsByPath('/tmp/a.txt')).toEqual([]);
    expect(store.getResultsByPath('/tmp/b.txt')).toHaveLength(1);
  });
});
