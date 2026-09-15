import App from '@app/App.vue';
import { mount } from '@vue/test-utils';
import { createPinia } from 'pinia';
import { describe, expect, it } from 'vitest';

describe('App.vue', () => {
  it('可以掛載完整應用程式', () => {
    const wrapper = mount(App, {
      global: {
        plugins: [createPinia()],
      },
    });

    expect(wrapper.exists()).toBe(true);
  });
});
