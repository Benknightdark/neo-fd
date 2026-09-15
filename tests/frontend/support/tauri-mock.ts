import type { Page } from '@playwright/test';

export interface E2eScanResult {
  path: string;
  line_num: number;
  pattern_name: string;
  matched_text: string;
}

export interface E2eFixture {
  files: Record<string, string>;
  results: E2eScanResult[];
}

export interface RecordedCommand {
  command: string;
  args: Record<string, unknown>;
}

interface E2eRuntimeState {
  commands: RecordedCommand[];
  files: Record<string, string>;
}

interface E2eWindow extends Window {
  __NEO_FD_E2E__?: E2eRuntimeState;
}

export async function installTauriMock(
  page: Page,
  fixture: E2eFixture,
): Promise<void> {
  await page.addInitScript(({ files, results }) => {
    type EventCallback = (event: {
      event: string;
      id: number;
      payload: unknown;
    }) => void;

    interface TauriInternals {
      callbacks: Map<number, EventCallback>;
      invoke: (
        command: string,
        args?: Record<string, unknown>,
      ) => Promise<unknown>;
      runCallback: (id: number, event: unknown) => void;
      transformCallback: (callback: EventCallback, once?: boolean) => number;
      unregisterCallback: (id: number) => void;
    }

    interface TauriEventInternals {
      unregisterListener: (event: string, id: number) => void;
    }

    const runtime: E2eRuntimeState = {
      commands: [],
      files: { ...files },
    };
    const callbacks = new Map<number, EventCallback>();
    const listeners = new Map<string, Set<number>>();
    let nextCallbackId = 1;

    function recordCommand(command: string, args: Record<string, unknown>) {
      runtime.commands.push({ command, args });
    }

    function emitEvent(eventName: string, payload: unknown) {
      const eventIds = listeners.get(eventName) ?? new Set<number>();
      for (const id of eventIds) {
        callbacks.get(id)?.({ event: eventName, id, payload });
      }
    }

    function removeCallback(id: number) {
      callbacks.delete(id);
      for (const eventIds of listeners.values()) {
        eventIds.delete(id);
      }
    }

    const invoke = async (
      command: string,
      args: Record<string, unknown> = {},
    ): Promise<unknown> => {
      recordCommand(command, args);

      switch (command) {
        case 'scan_directory': {
          const maxResults =
            typeof args.maxResults === 'number' ? args.maxResults : undefined;
          const batch =
            maxResults === undefined ? results : results.slice(0, maxResults);
          window.setTimeout(() => {
            emitEvent('scan-result-batch', batch);
            emitEvent('scan-finished', undefined);
          }, 10);
          return undefined;
        }
        case 'cancel_scan':
          emitEvent('scan-finished', undefined);
          return undefined;
        case 'read_file_content': {
          const path = String(args.path);
          if (!(path in runtime.files)) {
            throw '所選路徑不是有效的檔案。';
          }
          return runtime.files[path];
        }
        case 'write_file_content': {
          const path = String(args.path);
          if (!(path in runtime.files)) {
            throw '所選路徑的檔案不存在。';
          }
          runtime.files[path] = String(args.content);
          return undefined;
        }
        case 'delete_file': {
          const path = String(args.path);
          if (!(path in runtime.files)) {
            throw '檔案不存在或已被刪除。';
          }
          delete runtime.files[path];
          return undefined;
        }
        case 'plugin:event|listen': {
          const eventName = String(args.event);
          const callbackId = Number(args.handler);
          const eventIds = listeners.get(eventName) ?? new Set<number>();
          eventIds.add(callbackId);
          listeners.set(eventName, eventIds);
          return callbackId;
        }
        case 'plugin:event|unlisten': {
          removeCallback(Number(args.eventId));
          return undefined;
        }
        case 'plugin:event|emit':
          emitEvent(String(args.event), args.payload);
          return undefined;
        case 'plugin:dialog|open':
        case 'plugin:opener|open_path':
          return undefined;
        default:
          throw new Error(`未模擬的 IPC 指令: ${command}`);
      }
    };

    const tauriInternals: TauriInternals = {
      callbacks,
      invoke,
      runCallback: (id, event) => {
        callbacks.get(id)?.(event as Parameters<EventCallback>[0]);
      },
      transformCallback: (callback, once = false) => {
        const id = nextCallbackId;
        nextCallbackId += 1;
        callbacks.set(id, (event) => {
          if (once) {
            removeCallback(id);
          }
          callback(event);
        });
        return id;
      },
      unregisterCallback: removeCallback,
    };

    const testWindow = window as E2eWindow & {
      __TAURI_EVENT_PLUGIN_INTERNALS__?: TauriEventInternals;
      __TAURI_INTERNALS__?: TauriInternals;
    };
    testWindow.__NEO_FD_E2E__ = runtime;
    testWindow.__TAURI_INTERNALS__ = tauriInternals;
    testWindow.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: (_event, id) => removeCallback(id),
    };
  }, fixture);
}

export async function getRecordedCommands(
  page: Page,
): Promise<RecordedCommand[]> {
  return page.evaluate(() => {
    const runtime = (window as E2eWindow).__NEO_FD_E2E__;
    return runtime?.commands ?? [];
  });
}
