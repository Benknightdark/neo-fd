<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { useVirtualList } from '@vueuse/core';
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  ref,
  shallowRef,
} from 'vue';
import { type ScanResult, scannerApi, systemApi } from './api/ipc';

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
let batchTimer: ReturnType<typeof setInterval> | null = null;

// References to unlisten functions for garbage collection
let unlistenResult: (() => void) | null = null;
let unlistenFinished: (() => void) | null = null;

// Drawer state declarations
const isDrawerOpen = ref(false);
const isLoadingFile = ref(false);
const drawerPath = ref('');
const drawerTargetLine = ref<number | null>(null);
const drawerMatchedText = ref('');
const drawerActiveMatchIndex = ref(0);
const drawerLines = ref<string[]>([]);
const drawerError = ref('');

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

  // Defensive cleanup of any existing timer
  if (batchTimer) {
    clearInterval(batchTimer);
  }
  // Start batching timer
  batchTimer = setInterval(flushBuffer, 100);

  try {
    await scannerApi.startScan(scanPath.value || '/', activePatterns);
  } catch (err) {
    alert(`掃描啟動失敗: ${(err as Error).message}`);
    isScanning.value = false;
    if (batchTimer) {
      clearInterval(batchTimer);
      batchTimer = null;
    }
  }
}

// Computed active file matches from the master results list, sorted by line number
const activeFileMatches = computed(() => {
  if (!drawerPath.value) return [];
  return [...results.value]
    .filter((r) => r.path === drawerPath.value)
    .sort((a, b) => a.line_num - b.line_num);
});

// Open sliding drawer and load file contents
async function openDrawer(result: ScanResult) {
  isDrawerOpen.value = true;
  isLoadingFile.value = true;
  drawerPath.value = result.path;
  drawerTargetLine.value = result.line_num;
  drawerMatchedText.value = result.matched_text;
  drawerLines.value = [];
  drawerError.value = '';

  // Initialize active match index by exact match lookup
  const clickedIndex = activeFileMatches.value.findIndex(
    (r) =>
      r.line_num === result.line_num && r.matched_text === result.matched_text,
  );
  drawerActiveMatchIndex.value = clickedIndex !== -1 ? clickedIndex : 0;

  try {
    const content = await scannerApi.readFileContent(result.path);
    drawerLines.value = content.split(/\r?\n/);

    // Smooth scroll to the highlighted line after Vue updates the DOM
    await nextTick();
    const targetElement = document.querySelector('.highlight-line');
    if (targetElement) {
      targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  } catch (err) {
    drawerError.value = (err as Error).message || '無法載入檔案內容';
  } finally {
    isLoadingFile.value = false;
  }
}

function closeDrawer() {
  isDrawerOpen.value = false;
  // Delay clearing contents to let close transition play out nicely
  setTimeout(() => {
    if (!isDrawerOpen.value) {
      drawerLines.value = [];
      drawerPath.value = '';
      drawerTargetLine.value = null;
      drawerMatchedText.value = '';
      drawerActiveMatchIndex.value = 0;
      drawerError.value = '';
    }
  }, 400);
}

function getFileName(fullPath: string): string {
  const parts = fullPath.split(/[/\\]/);
  return parts[parts.length - 1] || fullPath;
}

// Strong-typed segment splitting interface
interface TextPart {
  text: string;
  isMatch: boolean;
}

// Safe substring splitting to isolate matched tokens without raw HTML
function splitMatchedLine(lineText: string, matchedText: string): TextPart[] {
  if (!matchedText) return [{ text: lineText, isMatch: false }];

  const parts: TextPart[] = [];
  let startIndex = 0;
  const matchLen = matchedText.length;

  while (true) {
    const matchIndex = lineText.indexOf(matchedText, startIndex);
    if (matchIndex === -1) {
      parts.push({ text: lineText.substring(startIndex), isMatch: false });
      break;
    }

    if (matchIndex > startIndex) {
      parts.push({
        text: lineText.substring(startIndex, matchIndex),
        isMatch: false,
      });
    }

    parts.push({ text: matchedText, isMatch: true });
    startIndex = matchIndex + matchLen;
  }

  return parts;
}

// Jump to a specific match index in the active file
async function jumpToMatch(index: number) {
  const targetResult = activeFileMatches.value[index];
  if (!targetResult) return;

  drawerActiveMatchIndex.value = index;
  drawerTargetLine.value = targetResult.line_num;
  drawerMatchedText.value = targetResult.matched_text;

  // Smooth scroll to the highlighted line after Vue updates the DOM
  await nextTick();
  const targetElement = document.querySelector('.highlight-line');
  if (targetElement) {
    targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

onMounted(async () => {
  // Store the unlisten handles to prevent memory leaks
  unlistenResult = await listen<ScanResult>('scan-result', (event) => {
    resultBuffer.push(event.payload);
  });

  unlistenFinished = await listen('scan-finished', () => {
    isScanning.value = false;
    flushBuffer(); // Final flush
    if (batchTimer) {
      clearInterval(batchTimer);
      batchTimer = null;
    }
    alert('掃描已完成');
  });
});

// Perform thorough cleanup on component unmount
onUnmounted(() => {
  if (unlistenResult) {
    unlistenResult();
  }
  if (unlistenFinished) {
    unlistenFinished();
  }
  if (batchTimer) {
    clearInterval(batchTimer);
    batchTimer = null;
  }
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
            <div 
              class="col path cursor-pointer text-blue-600 hover:underline" 
              :title="item.data.path"
              @click="openDrawer(item.data)"
            >
              {{ item.data.path }}
            </div>
            <div class="col line">{{ item.data.line_num }}</div>
            <div class="col matched">{{ item.data.matched_text }}</div>
          </div>
        </div>
        <div v-if="results.length === 0 && !isScanning" class="empty">
          尚未開始掃描或無結果
        </div>
      </div>
    </main>

    <!-- Drawer Slide-Over Panel -->
    <div class="drawer-container" :class="{ active: isDrawerOpen }">
      <div class="drawer-backdrop" @click="closeDrawer"></div>
      <aside class="drawer-panel">
        <header class="drawer-header">
          <div class="drawer-title-group">
            <h2>{{ getFileName(drawerPath) }}</h2>
            <span class="drawer-path" :title="drawerPath">{{ drawerPath }}</span>
          </div>
          <div class="drawer-actions">
            <button class="open-system-btn" title="使用系統預設應用程式開啟" @click="systemApi.openFile(drawerPath)">
              🗁 外部開啟
            </button>
            <button class="close-btn" @click="closeDrawer" aria-label="關閉面板">
              &times;
            </button>
          </div>
        </header>

        <div class="drawer-body">
          <!-- Loading State -->
          <div v-if="isLoadingFile" class="drawer-loading">
            <div class="spinner"></div>
            <p>載入檔案內容中...</p>
          </div>

          <!-- Error State -->
          <div v-else-if="drawerError" class="drawer-error">
            <div class="error-icon">⚠️</div>
            <p>{{ drawerError }}</p>
          </div>

          <!-- Code Viewer Content -->
          <div v-else class="code-viewer-container">
            <div class="code-header-info">
              <div class="code-stats">
                <span>高亮行號: <strong>{{ drawerTargetLine }}</strong></span>
                <span class="divider">|</span>
                <span>共 <strong>{{ drawerLines.length }}</strong> 行</span>
              </div>
              <div v-if="activeFileMatches.length > 1" class="navigator-group">
                <button 
                  class="nav-btn" 
                  :disabled="drawerActiveMatchIndex <= 0" 
                  @click="jumpToMatch(drawerActiveMatchIndex - 1)"
                  title="上一個個資洩漏點"
                >
                  ▲ 上一個
                </button>
                <span class="match-indicator">
                  第 {{ drawerActiveMatchIndex + 1 }} / {{ activeFileMatches.length }} 筆
                </span>
                <button 
                  class="nav-btn" 
                  :disabled="drawerActiveMatchIndex >= activeFileMatches.length - 1" 
                  @click="jumpToMatch(drawerActiveMatchIndex + 1)"
                  title="下一個個資洩漏點"
                >
                  下一個 ▼
                </button>
              </div>
            </div>
            <div class="code-viewer">
              <div 
                v-for="(line, index) in drawerLines" 
                :key="index" 
                class="code-line"
                :class="{ 'highlight-line': index + 1 === drawerTargetLine }"
              >
                <span class="line-number">{{ index + 1 }}</span>
                <pre class="line-content"><template v-if="index + 1 === drawerTargetLine && drawerMatchedText"><span v-for="(part, pIdx) in splitMatchedLine(line, drawerMatchedText)" :key="pIdx" :class="{ 'match-segment': part.isMatch }">{{ part.text }}</span></template><template v-else>{{ line || ' ' }}</template></pre>
              </div>
            </div>
          </div>
        </div>
      </aside>
    </div>
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

/* Drawer Container & Backdrop Styles */
.drawer-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 1000;
  pointer-events: none; /* Let clicks pass through when inactive */
}

.drawer-container.active {
  pointer-events: auto;
}

.drawer-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(4px);
  opacity: 0;
  transition: opacity 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  pointer-events: none;
}

.drawer-container.active .drawer-backdrop {
  opacity: 1;
  pointer-events: auto;
}

/* Drawer Panel Style - Premium Slide-Over */
.drawer-panel {
  position: absolute;
  top: 0;
  right: 0;
  width: 650px;
  max-width: 90vw;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  box-shadow: -10px 0 30px rgba(0, 0, 0, 0.15);
  border-left: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
  transform: translateX(100%);
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.drawer-container.active .drawer-panel {
  transform: translateX(0);
}

/* Drawer Header Styles */
.drawer-header {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #fafafa;
}

.drawer-title-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-width: 70%;
}

.drawer-title-group h2 {
  margin: 0;
  font-size: 1.25rem;
  color: #1a1a1a;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drawer-path {
  font-size: 0.8rem;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.drawer-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.open-system-btn {
  padding: 6px 12px;
  background: #e9ecef;
  color: #495057;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.open-system-btn:hover {
  background: #dee2e6;
  border-color: #adb5bd;
  color: #212529;
}

.close-btn {
  background: none;
  border: none;
  font-size: 2rem;
  color: #999;
  cursor: pointer;
  line-height: 1;
  padding: 4px 8px;
  border-radius: 50%;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #333;
  transform: scale(1.05);
}

/* Drawer Body Styles */
.drawer-body {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Loading & Error States */
.drawer-loading, .drawer-error {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  color: #666;
  gap: 16px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(0, 123, 255, 0.1);
  border-radius: 50%;
  border-top-color: #007bff;
  animation: spin 1s ease-in-out infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon {
  font-size: 3rem;
}

.drawer-error p {
  color: #dc3545;
  font-weight: 500;
  text-align: center;
}

/* Code Viewer Layout and Highlighting */
.code-viewer-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.code-header-info {
  padding: 8px 20px;
  background: #f1f3f5;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  font-size: 0.8rem;
  color: #495057;
  display: flex;
  justify-content: space-between;
  align-items: center; /* Vertical center alignment */
}

.code-stats {
  display: flex;
  align-items: center;
  gap: 8px;
}

.divider {
  color: #dee2e6;
  font-weight: 300;
}

/* Capsule Navigation HUD Styles */
.navigator-group {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 20px;
  padding: 2px 10px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.nav-btn {
  background: none;
  border: none;
  font-size: 0.75rem;
  color: #495057;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 10px;
  transition: all 0.2s ease;
  font-weight: bold;
}

.nav-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.05);
  color: #007bff;
}

.nav-btn:disabled {
  color: #ced4da;
  cursor: not-allowed;
}

.match-indicator {
  font-size: 0.75rem;
  color: #6c757d;
  font-weight: 500;
}

.code-viewer {
  flex: 1;
  overflow: auto;
  background: #2b2b2b; /* Sleek dark theme code block */
  color: #a9b7c6;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.85rem;
  padding: 10px 0;
}

.code-line {
  display: flex;
  min-height: 22px;
  line-height: 22px;
  width: 100%;
  white-space: pre;
}

.line-number {
  width: 50px;
  min-width: 50px;
  text-align: right;
  padding-right: 15px;
  color: #606366;
  user-select: none;
  background: rgba(0, 0, 0, 0.05);
  border-right: 1px solid rgba(255, 255, 255, 0.05);
}

.line-content {
  margin: 0;
  padding-left: 15px;
  flex: 1;
  overflow-x: auto;
  color: #e8e8e8;
}

/* Golden Highlight style for the matched leak point line */
.code-line.highlight-line {
  background: rgba(255, 193, 7, 0.15); /* Warm golden tint background */
  position: relative;
}

.code-line.highlight-line::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 4px;
  background: #ffc107; /* Vivid gold left border indicator */
  box-shadow: 0 0 8px #ffc107;
}

.code-line.highlight-line .line-number {
  color: #ffc107;
  background: rgba(255, 193, 7, 0.1);
  font-weight: bold;
}

.code-line.highlight-line .line-content {
  color: #fff;
  font-weight: 500;
}

/* Ruby red glow style for specific matched regex token segments */
.match-segment {
  background: rgba(220, 53, 69, 0.25); /* Ruby red transparent background */
  color: #ff4757; /* Bright soft red text */
  border: 1px solid rgba(220, 53, 69, 0.4);
  border-radius: 4px;
  padding: 1px 4px;
  font-weight: bold;
  box-shadow: 0 0 6px rgba(220, 53, 69, 0.2);
  text-shadow: 0 0 1px rgba(220, 53, 69, 0.1);
}
</style>
