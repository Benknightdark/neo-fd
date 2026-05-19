import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { useNotificationStore } from '../stores/notification';

/**
 * 標準 IPC 呼叫封裝
 *
 * 此工具統一處理來自 Rust 的 Result<T, E>。
 * 如果 Rust 傳回 Err，會自動發送通知到全域 Store 並拋出 Exception。
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (error) {
    const notification = useNotificationStore();
    console.error(`[IPC Error] Command: ${command}, Error:`, error);

    // 格式化錯誤訊息
    const message = typeof error === 'string' ? error : JSON.stringify(error);

    // 自動發送錯誤通知
    notification.add(`系統錯誤: ${message}`, 'error');

    throw new Error(message);
  }
}

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
    safeInvoke<void>('scan_directory', { path, patterns }),
};
