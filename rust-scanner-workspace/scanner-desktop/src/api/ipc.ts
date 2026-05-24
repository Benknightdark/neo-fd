import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { useNotificationStore } from '../stores/notification';

/**
 * 定義標準 Tauri 介面合約，確保前端與後端強烈綁定
 */
export interface TauriCommands {
  scan_directory: {
    args: { path: string; patterns: [string, string][] };
    // biome-ignore lint/suspicious/noConfusingVoidType: void is appropriate for representing a command with no return value
    return: void;
  };
  read_file_content: {
    args: { path: string };
    return: string;
  };
}

/**
 * 健全的錯誤格式化工具，解決 JSON.stringify(Error) => "{}" 的問題
 */
export function formatError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as Record<string, unknown>).message);
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

/**
 * 標準 IPC 呼叫封裝 (型別安全版)
 *
 * 此工具統一處理來自 Rust 的 Result<T, E>。
 * 如果 Rust 傳回 Err，會自動發送通知到全域 Store 並拋出 Exception。
 */
export async function safeInvoke<K extends keyof TauriCommands>(
  command: K,
  args: TauriCommands[K]['args'],
): Promise<TauriCommands[K]['return']> {
  try {
    return await tauriInvoke<TauriCommands[K]['return']>(command, args);
  } catch (error) {
    const notification = useNotificationStore();
    console.error(`[IPC Error] Command: ${command}, Error:`, error);

    // 格式化錯誤訊息
    const message = formatError(error);

    // 自動發送錯誤通知
    notification.add(`系統錯誤: ${message}`, 'error');

    throw new Error(message);
  }
}

/**
 * 系統層級 API 定義
 */
export const systemApi = {
  /**
   * 使用預設應用程式開啟檔案
   */
  openFile: async (path: string) => {
    try {
      await openPath(path);
    } catch (error) {
      const notification = useNotificationStore();
      const message = formatError(error);
      notification.add(`無法開啟檔案: ${message}`, 'warning');
      console.error(`[Open File Error] Path: ${path}, Error:`, error);
    }
  },
};

/**
 * 掃描相關的 API 定義
 */
export interface ScanResult {
  path: string;
  line_num: number;
  pattern_name: string;
  matched_text: string;
}

export const scannerApi = {
  startScan: (path: string, patterns: [string, string][]) =>
    safeInvoke('scan_directory', { path, patterns }),
  readFileContent: (path: string) => safeInvoke('read_file_content', { path }),
};
