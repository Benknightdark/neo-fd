import { listen } from '@tauri-apps/api/event';
import { defineStore } from 'pinia';
import { ref, shallowRef } from 'vue';
import { type ScanResult, scannerApi } from '../api/ipc';
import { useNotificationStore } from './notification';

// 核心掃描狀態 Store，處理掃描設定、批次 Buffer 與 Tauri IPC 監聽
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

  // 定期批次寫入緩衝區，避免大量即時 IPC 結果導致 UI 渲染卡頓
  let resultBuffer: ScanResult[] = [];
  let batchTimer: ReturnType<typeof setInterval> | null = null;

  // 儲存 Tauri 事件登出控制代碼，防止記憶體洩漏
  let unlistenResult: (() => void) | null = null;
  let unlistenFinished: (() => void) | null = null;

  // 將 Buffer 中的資料一次性寫入響應式 results，每 100ms 觸發一次
  function flushBuffer() {
    if (resultBuffer.length > 0) {
      results.value = [...results.value, ...resultBuffer];
      resultBuffer = [];
    }
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

    results.value = [];
    isScanning.value = true;

    // 清理舊定時器並重新啟動
    if (batchTimer) {
      clearInterval(batchTimer);
    }
    batchTimer = setInterval(flushBuffer, 100);

    try {
      // 呼叫 Rust 後端開始掃描
      await scannerApi.startScan(scanPath.value || '/', activePatterns);
    } catch (err) {
      isScanning.value = false;
      if (batchTimer) {
        clearInterval(batchTimer);
        batchTimer = null;
      }
    }
  }

  // 初始化註冊 Rust 後端的監聽器
  async function init() {
    if (unlistenResult || unlistenFinished) return;

    // 監聽單筆掃描結果事件，暫存於 Buffer 中
    unlistenResult = await listen<ScanResult>('scan-result', (event) => {
      resultBuffer.push(event.payload);
    });

    // 監聽掃描完畢事件，進行最終 Flush 與清理
    unlistenFinished = await listen('scan-finished', () => {
      isScanning.value = false;
      flushBuffer();
      if (batchTimer) {
        clearInterval(batchTimer);
        batchTimer = null;
      }
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
    if (batchTimer) {
      clearInterval(batchTimer);
      batchTimer = null;
    }
  }

  return {
    scanPath,
    isScanning,
    results,
    selectedPatterns,
    customPattern,
    customName,
    startScan,
    init,
    cleanup,
  };
});
