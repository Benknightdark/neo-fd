<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onMounted, ref } from 'vue';

interface ScanResult {
  path: string;
  line_num: number;
  pattern_name: string;
  matched_text: string;
}

const scanPath = ref('');
const isScanning = ref(false);
const results = ref<ScanResult[]>([]);
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

async function startScan() {
  if (isScanning.value) return;

  const activePatterns = selectedPatterns.value
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

  try {
    await invoke('scan_directory', {
      path: scanPath.value || '/',
      patterns: activePatterns,
    });
  } catch (err) {
    alert(`掃描啟動失敗: ${err}`);
    isScanning.value = false;
  }
}

onMounted(async () => {
  await listen<ScanResult>('scan-result', (event) => {
    results.value.push(event.payload);
  });

  await listen('scan-finished', () => {
    isScanning.value = false;
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
      
      <div class="table-container">
        <table>
          <thead>
            <tr>
              <th>類別</th>
              <th>檔案路徑</th>
              <th>行號</th>
              <th>匹配內容</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(res, index) in results" :key="index">
              <td><span class="badge">{{ res.pattern_name }}</span></td>
              <td class="path" :title="res.path">{{ res.path }}</td>
              <td>{{ res.line_num }}</td>
              <td class="text">{{ res.matched_text }}</td>
            </tr>
          </tbody>
        </table>
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

.table-container {
  flex: 1;
  overflow: auto;
  padding: 20px;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th, td {
  text-align: left;
  padding: 12px;
  border-bottom: 1px solid #eee;
  font-size: 0.9rem;
}

th {
  background: #fafafa;
  position: sticky;
  top: 0;
}

.badge {
  background: #e7f3ff;
  color: #007bff;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.8rem;
}

.path {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #666;
}

.text {
  color: #d63384;
  font-family: monospace;
}

.empty {
  text-align: center;
  margin-top: 50px;
  color: #999;
}
</style>
