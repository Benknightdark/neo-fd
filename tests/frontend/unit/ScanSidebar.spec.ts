import ScanSidebar from '@app/components/ScanSidebar.vue';
import { useScanStore } from '@app/stores/scan';
import { appDisplayVersion } from '@app/utils/appMeta';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';

describe('ScanSidebar', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('在標題與底部顯示環境與版號', () => {
    const wrapper = mount(ScanSidebar, {
      global: {
        plugins: [createPinia()],
      },
    });

    expect(
      wrapper.text().match(new RegExp(appDisplayVersion, 'g')),
    ).toHaveLength(2);
  });

  it('顯示搜尋開始與結束時間', () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const store = useScanStore();
    store.scanStartTime = new Date(2026, 5, 24, 15, 30, 12);
    store.scanEndTime = new Date(2026, 5, 24, 15, 45, 30);

    const wrapper = mount(ScanSidebar, {
      global: {
        plugins: [pinia],
      },
    });

    const text = wrapper.text();
    expect(text).toContain('開始時間');
    expect(text).toContain('結束時間');
    expect(text).toContain('2026/06/24');
  });
});
