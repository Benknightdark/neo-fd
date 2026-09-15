import { getAppEnvironmentLabel } from '@app/utils/appMeta';
import { describe, expect, it } from 'vitest';

describe('appMeta', () => {
  it('依 Vite 模式回傳正確的環境標籤', () => {
    expect(getAppEnvironmentLabel('development')).toBe('開發');
    expect(getAppEnvironmentLabel('production')).toBe('正式');
    expect(getAppEnvironmentLabel('test')).toBe('正式');
  });
});
