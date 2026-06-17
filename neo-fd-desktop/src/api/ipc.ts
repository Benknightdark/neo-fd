import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { openPath } from '@tauri-apps/plugin-opener';
import { useNotificationStore } from '../stores/notification';

/**
 * 定義與 Rust 後端綁定的強型別 Tauri 指令契約
 */
export interface TauriCommands {
  // 啟動目錄掃描
  scan_directory: {
    args: {
      path: string;
      patterns: [string, string][];
      maxResults: number | null;
    };
    // biome-ignore lint/suspicious/noConfusingVoidType: void represents command without return value
    return: void;
  };
  // 中止目錄掃描
  cancel_scan: {
    args: Record<string, never>;
    // biome-ignore lint/suspicious/noConfusingVoidType: void represents command without return value
    return: void;
  };
  // 讀取指定檔案內容
  read_file_content: {
    args: { path: string };
    return: string;
  };
  // 寫入指定檔案內容
  write_file_content: {
    args: { path: string; content: string };
    // biome-ignore lint/suspicious/noConfusingVoidType: void represents command without return value
    return: void;
  };
  // 刪除指定檔案
  delete_file: {
    args: { path: string };
    // biome-ignore lint/suspicious/noConfusingVoidType: void represents command without return value
    return: void;
  };
}

/**
 * 格式化與標準化各種類型的錯誤訊息，確保錯誤可見度
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
 * 強型別安全的 Tauri IPC 呼叫代理
 * 自動攔截執行時異常並派送 Toast 全域通知，防止程序崩潰
 */
export async function safeInvoke<K extends keyof TauriCommands>(
  command: K,
  args: TauriCommands[K]['args'],
): Promise<TauriCommands[K]['return']> {
  try {
    return await tauriInvoke<TauriCommands[K]['return']>(command, args);
  } catch (error) {
    const notification = useNotificationStore();
    console.error(`[IPC 呼叫失敗] 指令: ${command}, 錯誤詳情:`, error);

    // 格式化並派送 Toast 視覺警告
    const message = formatError(error);
    notification.add(`系統錯誤: ${message}`, 'error');

    throw new Error(message);
  }
}

/**
 * 系統層級的桌面整合 API 封裝
 */
export const systemApi = {
  /**
   * 呼叫作業系統預設應用程式開啟指定檔案
   */
  openFile: async (path: string) => {
    try {
      await openPath(path);
    } catch (error) {
      const notification = useNotificationStore();
      const message = formatError(error);
      notification.add(`無法開啟檔案: ${message}`, 'warning');
      console.error(`[外部開啟失敗] 路徑: ${path}, 錯誤詳情:`, error);
    }
  },
};

/**
 * 掃描匹配結果的強型別資料結構
 */
export interface ScanResult {
  path: string; // 檔案路徑
  line_num: number; // 匹配行號
  pattern_name: string; // 比對規則名稱
  matched_text: string; // 敏感字串片段
}

/**
 * 掃描業務領域 IPC 介面
 */
export const scannerApi = {
  // 開始目錄掃描
  startScan: (
    path: string,
    patterns: [string, string][],
    maxResults: number | null,
  ) => safeInvoke('scan_directory', { path, patterns, maxResults }),
  // 中止目錄掃描
  cancelScan: () => safeInvoke('cancel_scan', {}),
  // 載入特定檔案內容以供 Code Viewer 渲染
  readFileContent: (path: string) => safeInvoke('read_file_content', { path }),
  // 寫入檔案內容以進行修正
  writeFileContent: (path: string, content: string) =>
    safeInvoke('write_file_content', { path, content }),
  // 永久刪除洩漏檔案
  deleteFile: (path: string) => safeInvoke('delete_file', { path }),
};
