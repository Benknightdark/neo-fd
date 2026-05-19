<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { useVirtualList } from '@vueuse/core';
import { onMounted, ref, shallowRef } from 'vue';
import { type ScanResult, scannerApi } from './api/ipc';

const scanPath = ref('');
const isScanning = ref(false);
const results = shallowRef<ScanResult[]>([]);
const { list, containerProps, wrapperProps } = useVirtualList(results, {
  itemHeight: 45, // Match row height
});

const selectedPatterns = ref([
  { name: '身分證字號', pattern: '[A-Za-z][12]\\d{8}', enabled: true },
  {
    name: '台灣十大姓氏',
    pattern: '[陳林黃張李王吳劉蔡楊][\u4e00-\u9fa5]{2}',
    enabled: true,
  },
]);
const customPattern = ref('');
const customName = ref('自定義');

// Buffer logic for batching updates
let resultBuffer: ScanResult[] = [];
let batchTimer: number | null = null;

function flushBuffer() {
  if (resultBuffer.length > 0) {
    results.value = [...results.value, ...resultBuffer];
    resultBuffer = [];
  }
}

async function startScan() {
  if (isScanning.value) return;

  const activePatterns: [string, string][] = selectedPatterns.value
    .filter((p) => p.enabled)
    .map((p) => [p.name, p.pattern]);

  if (customPattern.value) {
    activePatterns.push([customName.value, customPattern.value]);
  }

  if (activePatterns.length === 0) {
    alert('請至少選擇或輸入一個正則表達式');
    return;
  }

  results.value = [];
  isScanning.value = true;

  // Start batching timer
  batchTimer = window.setInterval(flushBuffer, 100);

  try {
    await scannerApi.startScan(scanPath.value || '/', activePatterns);
  } catch (err) {
    alert(`掃描啟動失敗: ${(err as Error).message}`);
    isScanning.value = false;
    if (batchTimer) clearInterval(batchTimer);
  }
}

onMounted(async () => {
  await listen<ScanResult>('scan-result', (event) => {
    resultBuffer.push(event.payload);
  });

  await listen('scan-finished', () => {
    isScanning.value = false;
    flushBuffer(); // Final flush
    if (batchTimer) {
      clearInterval(batchTimer);
      batchTimer = null;
    }
    alert('掃描已完成');
  });
});
</script>

<template>
  <div class="container">
    <aside class="sidebar">
      <h2>掃描設定</h2>
      <div class="field">
        <label>目標路徑</label>
        <input v-model="scanPath" placeholder="例如: /Users/name/Documents" />
      </div>
      
      <div class="field">
        <label>內建模式</label>
        <div v-for="p in selectedPatterns" :key="p.name" class="checkbox-group">
          <input type="checkbox" v-model="p.enabled" :id="p.name" />
          <label :for="p.name">{{ p.name }}</label>
        </div>
      </div>
      
      <div class="field">
        <label>自定義 Regex</label>
        <input v-model="customPattern" placeholder="例如: [0-9]+" />
      </div>
      
      <button @click="startScan" :disabled="isScanning">
        {{ isScanning ? '掃描中...' : '開始掃描' }}
      </button>
    </aside>
    
    <main class="content">
      <header>
        <h1>掃描結果 ({{ results.length }})</h1>
      </header>
      
      <div class="table-header">
        <div class="col type">類別</div>
        <div class="col path">檔案路徑</div>
        <div class="col line">行號</div>
        <div class="col matched">匹配內容</div>
      </div>

      <div v-bind="containerProps" class="table-container">
        <div v-bind="wrapperProps">
          <div v-for="item in list" :key="item.index" class="table-row" :style="{ height: '45px' }">
            <div class="col type"><span class="badge">{{ item.data.pattern_name }}</span></div>
            <div class="col path" :title="item.data.path">{{ item.data.path }}</div>
            <div class="col line">{{ item.data.line_num }}</div>
            <div class="col matched">{{ item.data.matched_text }}</div>
          </div>
        </div>
        <div v-if="results.length === 0 && !isScanning" class="empty">
          尚未開始掃描或無結果
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.container {
  display: flex;
  height: 100vh;
  font-family: system-ui, -apple-system, sans-serif;
  color: #333;
}

.sidebar {
  width: 250px;
  background: #f4f4f9;
  padding: 20px;
  border-right: 1px solid #ddd;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field label {
  font-weight: bold;
  font-size: 0.9rem;
}

input[type="text"], input:not([type]) {
  padding: 8px;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.checkbox-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
}

button {
  padding: 10px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

button:disabled {
  background: #ccc;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

header {
  padding: 0 20px;
  border-bottom: 1px solid #eee;
}

.table-header {
  display: flex;
  background: #fafafa;
  border-bottom: 1px solid #eee;
  padding: 0 20px;
  font-weight: bold;
  font-size: 0.9rem;
}

.table-row {
  display: flex;
  align-items: center;
  padding: 0 20px;
  border-bottom: 1px solid #f9f9f9;
  font-size: 0.9rem;
}

.table-container {
  flex: 1;
  overflow: auto;
}

.col {
  padding: 12px 5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.type { width: 120px; }
.path { flex: 1; color: #666; }
.line { width: 80px; }
.matched { width: 250px; color: #d63384; font-family: monospace; }

.badge {
  background: #e7f3ff;
  color: #007bff;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.8rem;
}

.empty {
  text-align: center;
  margin-top: 50px;
  color: #999;
}
</style>
