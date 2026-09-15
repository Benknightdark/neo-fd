import { formatError } from '@app/api/ipc';
import { describe, expect, it } from 'vitest';

describe('formatError', () => {
  it('保留字串錯誤訊息', () => {
    expect(formatError('發生錯誤')).toBe('發生錯誤');
  });

  it('轉換常見錯誤物件', () => {
    expect(formatError(new Error('讀取失敗'))).toBe('讀取失敗');
    expect(formatError({ message: '命令失敗' })).toBe('命令失敗');
  });
});
