<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import { computed } from 'vue';
import { useScanStore } from '../stores/scan';
import { appDisplayVersion } from '../utils/appMeta';

const store = useScanStore();

const scanTimeFormatter = new Intl.DateTimeFormat('zh-TW', {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
  second: '2-digit',
  hour12: false,
});

function formatScanTime(value: Date | null): string {
  return value ? scanTimeFormatter.format(value) : '尚未記錄';
}

const scanStartTimeText = computed(() => formatScanTime(store.scanStartTime));
const scanEndTimeText = computed(() => formatScanTime(store.scanEndTime));

// 計算屬性守衛：確保至少選擇一個內建模式，或輸入自定義模式才可進行掃描
const canScan = computed(() => {
  if (store.isScanning) return false;

  const hasBuiltin = store.selectedPatterns.some((p) => p.enabled);
  const hasCustom = !!store.customPattern;

  return hasBuiltin || hasCustom;
});

// 觸發掃描程序
function handleScan() {
  if (!canScan.value) return;
  store.startScan();
}

// 開啟資料夾選擇對話框並更新掃描路徑
async function handleSelectDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: store.scanPath || undefined,
      title: '選擇掃描目標資料夾',
    });
    if (selected) {
      store.scanPath = Array.isArray(selected) ? selected[0] : selected;
    }
  } catch (err) {
    console.error('無法選取資料夾:', err);
  }
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="app-title-row">
        <h2>Neo FD</h2>
        <span class="version-badge">{{ appDisplayVersion }}</span>
      </div>
      <p class="sidebar-subtitle">掃描設定</p>
    </div>

    <div class="field">
      <label for="scan-path">目標路徑</label>
      <div class="input-with-button">
        <input
          id="scan-path"
          v-model="store.scanPath"
          type="text"
          placeholder="例如: /Users/name/Documents"
          :disabled="store.isScanning"
        />
        <button
          type="button"
          class="select-dir-btn"
          @click="handleSelectDir"
          :disabled="store.isScanning"
          title="選擇資料夾"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="folder-icon"
          >
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
        </button>
      </div>
    </div>

    <div class="field">
      <label>內建模式</label>
      <div class="patterns-list">
        <div
          v-for="p in store.selectedPatterns"
          :key="p.name"
          class="checkbox-group"
        >
          <input
            type="checkbox"
            v-model="p.enabled"
            :id="p.name"
            :disabled="store.isScanning"
          />
          <label :for="p.name">{{ p.name }}</label>
        </div>
      </div>
    </div>

    <div class="field">
      <label for="custom-regex">自定義 Regex</label>
      <div class="input-wrapper">
        <input
          id="custom-regex"
          v-model="store.customPattern"
          type="text"
          placeholder="例如: [0-9]+"
          :disabled="store.isScanning"
        />
      </div>
    </div>

    <div class="field">
      <label for="max-results">最大匹配筆數</label>
      <div class="input-wrapper">
        <input
          id="max-results"
          v-model="store.maxResultsInput"
          type="number"
          min="1"
          step="1"
          inputmode="numeric"
          placeholder="留空代表不限"
          :disabled="store.isScanning"
        />
      </div>
    </div>

    <div
      v-if="store.scanStartTime || store.scanEndTime"
      class="scan-time-panel"
    >
      <div v-if="store.scanStartTime" class="scan-time-row">
        <span class="scan-time-label">開始時間</span>
        <span class="scan-time-value">{{ scanStartTimeText }}</span>
      </div>
      <div v-if="store.scanEndTime" class="scan-time-row">
        <span class="scan-time-label">結束時間</span>
        <span class="scan-time-value">{{ scanEndTimeText }}</span>
      </div>
    </div>

    <button
      v-if="store.isScanning"
      class="scan-btn cancel-btn"
      @click="store.cancelScan"
    >
      <span class="spinner"></span>
      停止掃描
    </button>
    <button
      v-else
      class="scan-btn"
      @click="handleScan"
      :disabled="!canScan"
    >
      開始掃描
    </button>

    <footer class="sidebar-footer">
      {{ appDisplayVersion }}
    </footer>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 280px;
  background: #fafafc;
  padding: 24px;
  border-right: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
  gap: 24px;
  box-shadow: 2px 0 10px rgba(0, 0, 0, 0.01);
}

.sidebar-header h2 {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 700;
  color: #2c3e50;
  letter-spacing: 0.5px;
}

.app-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.version-badge {
  flex-shrink: 0;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(0, 123, 255, 0.08);
  color: #007bff;
  font-size: 0.68rem;
  font-weight: 700;
  white-space: nowrap;
}

.sidebar-subtitle {
  margin: 4px 0 0;
  font-size: 0.78rem;
  font-weight: 700;
  color: #7f8c8d;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field label {
  font-weight: 600;
  font-size: 0.85rem;
  color: #7f8c8d;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

input[type="text"],
input[type="number"] {
  padding: 10px 12px;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  font-size: 0.9rem;
  color: #2c3e50;
  background: #fff;
  transition: all 0.2s ease-in-out;
  width: 100%;
  box-sizing: border-box;
}

input[type="text"]:focus,
input[type="number"]:focus {
  border-color: #007bff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.15);
  outline: none;
}

input[type="text"]:disabled,
input[type="number"]:disabled {
  background: #f8f9fa;
  color: #adb5bd;
  cursor: not-allowed;
}

.input-with-button {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.input-with-button input[type="text"] {
  flex: 1;
  min-width: 0;
}

.select-dir-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  background: #fff;
  cursor: pointer;
  transition: all 0.2s ease-in-out;
  color: #7f8c8d;
  flex-shrink: 0;
}

.select-dir-btn:hover:not(:disabled) {
  border-color: #007bff;
  color: #007bff;
  background: #f0f7ff;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 123, 255, 0.1);
}

.select-dir-btn:active:not(:disabled) {
  transform: translateY(0);
}

.select-dir-btn:disabled {
  background: #f8f9fa;
  color: #adb5bd;
  cursor: not-allowed;
  border-color: #e9ecef;
}

.folder-icon {
  width: 18px;
  height: 18px;
}

.patterns-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: #fff;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #dee2e6;
}

.checkbox-group {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.9rem;
  color: #34495e;
  cursor: pointer;
}

.checkbox-group input[type="checkbox"] {
  cursor: pointer;
  width: 16px;
  height: 16px;
}

.scan-time-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  background: #fff;
}

.scan-time-row {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.scan-time-label {
  font-size: 0.7rem;
  font-weight: 700;
  color: #7f8c8d;
  letter-spacing: 0.4px;
}

.scan-time-value {
  color: #2c3e50;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.78rem;
  font-weight: 600;
}

.checkbox-group label {
  text-transform: none;
  font-weight: 500;
  color: #34495e;
  cursor: pointer;
}

.scan-btn {
  margin-top: auto;
  padding: 12px;
  background: linear-gradient(135deg, #007bff, #0056b3);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.95rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.2);
  transition: all 0.2s;
}

.scan-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #0069d9, #004085);
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(0, 123, 255, 0.3);
}

.scan-btn:disabled {
  background: #e9ecef;
  color: #ced4da;
  cursor: not-allowed;
  box-shadow: none;
}

.scan-btn.cancel-btn {
  background: linear-gradient(135deg, #dc3545, #bd2130);
  box-shadow: 0 4px 12px rgba(220, 53, 69, 0.25);
}

.scan-btn.cancel-btn:hover {
  background: linear-gradient(135deg, #c82333, #bd2130);
  box-shadow: 0 6px 16px rgba(220, 53, 69, 0.35);
}

.sidebar-footer {
  padding-top: 2px;
  color: #95a5a6;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-align: center;
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: #fff;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

</style>
