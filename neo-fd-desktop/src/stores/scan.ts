import { listen } from '@tauri-apps/api/event';
import { defineStore } from 'pinia';
import { ref, shallowRef } from 'vue';
import { type ScanResult, scannerApi } from '../api/ipc';
import { useNotificationStore } from './notification';

// 核心掃描狀態 Store，處理掃描設定、Tauri IPC 監聽與中止機制
export const useScanStore = defineStore('scan', () => {
  // 掃描目標目錄路徑
  const scanPath = ref('');

  // 是否正在掃描中
  const isScanning = ref(false);

  // 掃描結果列表，採用 shallowRef 提升大數據效能，避免深層 Reactivity 效能開銷
  const results = shallowRef<ScanResult[]>([]);

  // 預設內建的比對規則模式
  const selectedPatterns = ref([
    { name: '身分證字號', pattern: '[A-Za-z][12]\\d{8}', enabled: true },
    {
      name: '台灣十大姓氏',
      pattern: '[陳林黃張李王吳劉蔡楊][\u4e00-\u9fa5]{2}',
      enabled: true,
    },
  ]);

  // 使用者自定義 Regex 比對規則
  const customPattern = ref('');
  const customName = ref('自定義');

  // 最大匹配結果數；空字串代表不限制
  const maxResultsInput = ref('');

  // 儲存 Tauri 事件登出控制代碼，防止記憶體洩漏
  let unlistenResult: (() => void) | null = null;
  let unlistenFinished: (() => void) | null = null;

  function normalizeMaxResults(value: string): number | null {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }

    const maxResults = Number(trimmed);
    if (!Number.isInteger(maxResults) || maxResults <= 0) {
      throw new Error('最大匹配筆數必須是大於 0 的整數');
    }

    return maxResults;
  }

  // 啟動掃描程序
  async function startScan() {
    if (isScanning.value) return;

    // 彙整啟用的比對規則
    const activePatterns: [string, string][] = selectedPatterns.value
      .filter((p) => p.enabled)
      .map((p) => [p.name, p.pattern]);

    if (customPattern.value) {
      activePatterns.push([customName.value, customPattern.value]);
    }

    // 防禦阻擋：至少須啟用一項規則
    if (activePatterns.length === 0) {
      const notification = useNotificationStore();
      notification.add('請至少選擇或輸入一個正則表達式', 'warning');
      return;
    }

    const notification = useNotificationStore();
    let maxResults: number | null;
    try {
      maxResults = normalizeMaxResults(maxResultsInput.value);
    } catch (err) {
      notification.add((err as Error).message, 'warning');
      return;
    }

    results.value = [];
    isScanning.value = true;

    try {
      // 呼叫 Rust 後端開始掃描
      await scannerApi.startScan(
        scanPath.value || '/',
        activePatterns,
        maxResults,
      );
    } catch (err) {
      isScanning.value = false;
    }
  }

  // 中止掃描程序
  async function cancelScan() {
    if (!isScanning.value) return;
    try {
      await scannerApi.cancelScan();
      const notification = useNotificationStore();
      notification.add('正在中止掃描...', 'warning');
    } catch (err) {
      console.error('無法中止掃描:', err);
    }
  }

  // 初始化註冊 Rust 後端的監聽器
  async function init() {
    if (unlistenResult || unlistenFinished) return;

    // 監聽後端批次掃描結果事件 (傳入資料為 ScanResult 陣列)
    unlistenResult = await listen<ScanResult[]>(
      'scan-result-batch',
      (event) => {
        results.value = [...results.value, ...event.payload];
      },
    );

    // 監聽掃描完畢事件
    unlistenFinished = await listen('scan-finished', () => {
      isScanning.value = false;
      const notification = useNotificationStore();
      notification.add('掃描已完成', 'success');
    });
  }

  // 登出監聽器，釋放資源
  function cleanup() {
    if (unlistenResult) {
      unlistenResult();
      unlistenResult = null;
    }
    if (unlistenFinished) {
      unlistenFinished();
      unlistenFinished = null;
    }
  }

  // 根據檔案路徑將掃描結果從 results 中過濾，用於檔案刪除後的即時 UI 同步
  function removeResultsByPath(path: string) {
    results.value = results.value.filter((r) => r.path !== path);
  }

  return {
    scanPath,
    isScanning,
    results,
    selectedPatterns,
    customPattern,
    customName,
    maxResultsInput,
    normalizeMaxResults,
    startScan,
    cancelScan,
    init,
    cleanup,
    removeResultsByPath,
  };
});
