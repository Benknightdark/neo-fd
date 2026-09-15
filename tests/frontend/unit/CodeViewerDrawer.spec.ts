import CodeViewerDrawer from '@app/components/CodeViewerDrawer.vue';
import { mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { afterEach, describe, expect, it } from 'vitest';

const initialInnerWidth = window.innerWidth;

function setViewportWidth(width: number) {
  Object.defineProperty(window, 'innerWidth', {
    configurable: true,
    value: width,
  });
}

function mountDrawer() {
  return mount(CodeViewerDrawer, {
    props: {
      isOpen: false,
      activeResult: null,
    },
    global: {
      plugins: [createPinia()],
    },
  });
}

describe('CodeViewerDrawer', () => {
  afterEach(() => {
    setViewportWidth(initialInnerWidth);
  });

  it('開啟時使用當前畫面寬度的三分之二', async () => {
    setViewportWidth(1200);
    const wrapper = mountDrawer();

    await wrapper.setProps({ isOpen: true });

    expect(wrapper.find<HTMLElement>('.drawer-panel').element.style.width).toBe(
      '800px',
    );
    wrapper.unmount();
  });

  it('重新開啟時依最新畫面寬度重新計算', async () => {
    setViewportWidth(1200);
    const wrapper = mountDrawer();

    await wrapper.setProps({ isOpen: true });
    expect(wrapper.find<HTMLElement>('.drawer-panel').element.style.width).toBe(
      '800px',
    );

    await wrapper.setProps({ isOpen: false });
    setViewportWidth(1500);
    await wrapper.setProps({ isOpen: true });

    expect(wrapper.find<HTMLElement>('.drawer-panel').element.style.width).toBe(
      '1000px',
    );
    wrapper.unmount();
  });
});
