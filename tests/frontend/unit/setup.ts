import { afterEach, vi } from 'vitest';

type EventHandler = (event: { payload: unknown }) => void;

const mocks = vi.hoisted(() => {
  const eventListeners = new Map<string, EventHandler>();

  return {
    eventListeners,
    invoke: vi.fn(),
    listen: vi.fn((eventName: string, handler: EventHandler) => {
      eventListeners.set(eventName, handler);
      return Promise.resolve(() => {
        eventListeners.delete(eventName);
      });
    }),
    open: vi.fn(),
    openPath: vi.fn(),
  };
});

const localStorageMock = {
  clear: vi.fn(),
  getItem: vi.fn(() => null),
  removeItem: vi.fn(),
  setItem: vi.fn(),
};

vi.stubGlobal('localStorage', localStorageMock);
Object.defineProperty(window, 'localStorage', {
  configurable: true,
  value: localStorageMock,
});

export const eventListeners = mocks.eventListeners;
export const tauriInvoke = mocks.invoke;

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mocks.invoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: mocks.listen,
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: mocks.open,
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openPath: mocks.openPath,
}));

afterEach(() => {
  eventListeners.clear();
  mocks.invoke.mockReset();
  mocks.listen.mockClear();
  mocks.open.mockReset();
  mocks.openPath.mockReset();
  window.localStorage.clear();
});
