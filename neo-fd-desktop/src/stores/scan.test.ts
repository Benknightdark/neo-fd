import { invoke } from '@tauri-apps/api/core';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useScanStore } from './scan';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('useScanStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(invoke).mockClear();
    vi.mocked(invoke).mockResolvedValue(undefined);
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
});
