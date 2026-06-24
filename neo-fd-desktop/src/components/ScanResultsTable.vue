<script setup lang="ts">
import { useVirtualList } from '@vueuse/core';
import { storeToRefs } from 'pinia';
import { computed, onUnmounted, ref, shallowRef, watch } from 'vue';
import type { ScanResult } from '../api/ipc';
import { type ScanResultItem, useScanStore } from '../stores/scan';
import { buildResultTreeRows, type TreeNode } from '../utils/resultTree';

const RESULT_ROW_HEIGHT = 45;
const TREE_REBUILD_INTERVAL_MS = 250;

const store = useScanStore();
const { isScanning, results, resultsVersion } = storeToRefs(store);

// 宣告開口事件：當使用者點擊結果檔案時，發送該洩漏項目給父元件以開啟抽屜
const emit = defineEmits<(e: 'open-drawer', result: ScanResult) => void>();

// 檢視模式狀態管理，預設為 tree (樹狀檢視)，並使用 localStorage 記憶使用者選擇
type ViewMode = 'list' | 'tree';
const savedViewMode = localStorage.getItem(
  'neo-fd-view-mode',
) as ViewMode | null;
const viewMode = ref<ViewMode>(savedViewMode || 'tree');

watch(viewMode, (newMode) => {
  localStorage.setItem('neo-fd-view-mode', newMode);
});

// 記錄被折疊的節點 ID
const collapsedIds = ref<Set<string>>(new Set());
const treeRows = shallowRef<TreeNode[]>([]);
let treeRebuildTimer: ReturnType<typeof setTimeout> | null = null;
let lastTreeRebuildAt = 0;

function cancelScheduledTreeRebuild() {
  if (treeRebuildTimer === null) return;
  clearTimeout(treeRebuildTimer);
  treeRebuildTimer = null;
}

function rebuildTreeRows() {
  cancelScheduledTreeRebuild();
  treeRows.value = buildResultTreeRows(results.value, collapsedIds.value);
  lastTreeRebuildAt = Date.now();
}

function scheduleTreeRebuild(force = false) {
  if (viewMode.value !== 'tree') return;

  if (force || !isScanning.value) {
    rebuildTreeRows();
    return;
  }

  const elapsed = Date.now() - lastTreeRebuildAt;
  if (elapsed >= TREE_REBUILD_INTERVAL_MS) {
    rebuildTreeRows();
    return;
  }

  if (treeRebuildTimer !== null) return;
  treeRebuildTimer = setTimeout(
    rebuildTreeRows,
    TREE_REBUILD_INTERVAL_MS - elapsed,
  );
}

// 一鍵展開全部
function expandAll() {
  collapsedIds.value = new Set();
  scheduleTreeRebuild(true);
}

// 一鍵折疊全部
function collapseAll() {
  const allIds = new Set<string>();
  treeRows.value.forEach((node) => {
    if (node.type !== 'match') {
      allIds.add(node.id);
    }
  });
  collapsedIds.value = allIds;
  scheduleTreeRebuild(true);
}

// 切換單一節點摺疊狀態
function toggleNode(node: TreeNode) {
  if (node.type === 'match') return;
  const newCollapsed = new Set(collapsedIds.value);
  if (newCollapsed.has(node.id)) {
    newCollapsed.delete(node.id);
  } else {
    newCollapsed.add(node.id);
  }
  collapsedIds.value = newCollapsed;
  scheduleTreeRebuild(true);
}

watch(
  [viewMode, resultsVersion],
  () => {
    scheduleTreeRebuild();
  },
  { immediate: true },
);

watch(isScanning, (scanning, wasScanning) => {
  if (!scanning && wasScanning) {
    scheduleTreeRebuild(true);
  }
});

// 統合成單一虛擬滾動列表資料來源，依檢視模式切換
type DisplayRow = ScanResultItem | TreeNode;

const displayRows = computed<DisplayRow[]>(() => {
  if (viewMode.value === 'list') {
    return results.value;
  }
  return treeRows.value;
});

// 註冊高性能虛擬滾動列表
const { list, containerProps, wrapperProps } = useVirtualList(displayRows, {
  itemHeight: RESULT_ROW_HEIGHT,
  overscan: 12,
});

function isResultRow(row: DisplayRow): row is ScanResultItem {
  return row.rowKind === 'result';
}

function displayRowKey(row: DisplayRow): string {
  return row.rowKind === 'result' ? `result:${row.id}` : `tree:${row.id}`;
}

// 樹狀節點點擊邏輯
function handleNodeClick(node: TreeNode) {
  if (node.type === 'match') {
    if (node.scanResult) {
      emit('open-drawer', node.scanResult);
    }
  } else {
    toggleNode(node);
  }
}

onUnmounted(() => {
  cancelScheduledTreeRebuild();
});
</script>

<template>
  <main class="content">
    <header class="content-header">
      <div class="header-info">
        <h1>掃描結果</h1>
        <span class="count-badge">{{ results.length }} 筆</span>
      </div>

      <div class="header-actions">
        <!-- 檢視切換分段控制器 -->
        <div class="view-toggle">
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'tree' }"
            @click="viewMode = 'tree'"
            title="樹狀階層檢視"
          >
            🌿 樹狀檢視
          </button>
          <button
            class="toggle-btn"
            :class="{ active: viewMode === 'list' }"
            @click="viewMode = 'list'"
            title="平鋪列表檢視"
          >
            📋 列表檢視
          </button>
        </div>

        <!-- 樹狀控制輔助按鈕 -->
        <div v-if="viewMode === 'tree' && results.length > 0" class="tree-controls">
          <button class="control-btn" @click="expandAll" title="展開所有層級">
            📂 展開全部
          </button>
          <button class="control-btn" @click="collapseAll" title="收合所有層級">
            📁 折疊全部
          </button>
        </div>
      </div>
    </header>

    <div class="table-header" :class="{ 'is-tree': viewMode === 'tree' }">
      <template v-if="viewMode === 'list'">
        <div class="col type">類別</div>
        <div class="col path">檔案路徑</div>
        <div class="col line">行號</div>
        <div class="col matched">匹配內容</div>
      </template>
      <template v-else>
        <div class="col tree-header">檔案階層與敏感資料匹配內容</div>
      </template>
    </div>

    <div v-bind="containerProps" class="table-container">
      <div v-bind="wrapperProps">
        <div
          v-for="item in list"
          :key="displayRowKey(item.data)"
          class="table-row"
          :class="[
            isResultRow(item.data) ? 'list-row' : 'tree-row',
            !isResultRow(item.data) ? `node-${item.data.type}` : ''
          ]"
          :style="{ height: `${RESULT_ROW_HEIGHT}px` }"
        >
          <!-- A. 列表檢視渲染 -->
          <template v-if="isResultRow(item.data)">
            <div class="col type">
              <span class="badge">{{ item.data.pattern_name }}</span>
            </div>
            <div
              class="col path"
              :title="item.data.path"
              @click="emit('open-drawer', item.data)"
            >
              {{ item.data.path }}
            </div>
            <div class="col line">{{ item.data.line_num }}</div>
            <div class="col matched">{{ item.data.matched_text }}</div>
          </template>

          <!-- B. 樹狀檢視渲染 (Multi-stage Folder/File/Match nodes) -->
          <template v-else>
            <div
              class="tree-node-content"
              :style="{ paddingLeft: `${item.data.depth * 20 + 20}px` }"
              @click="handleNodeClick(item.data)"
            >
              <!-- 展開/收合狀態旋轉箭頭 -->
              <span
                v-if="item.data.type !== 'match'"
                class="chevron-icon"
                :class="{ 'is-open': item.data.isOpen }"
              >
                ▼
              </span>
              <span v-else class="chevron-placeholder"></span>

              <!-- 語意化圖示 -->
              <span class="node-icon">
                <template v-if="item.data.type === 'folder'">
                  {{ item.data.isOpen ? '📂' : '📁' }}
                </template>
                <template v-else-if="item.data.type === 'file'">
                  📄
                </template>
                <template v-else>
                  🔍
                </template>
              </span>

              <!-- 內容顯示 -->
              <span class="node-name" :class="{ 'is-match-text': item.data.type === 'match' }">
                <!-- 資料夾或檔案 -->
                <template v-if="item.data.type !== 'match'">
                  <span class="name-text" :title="item.data.name">{{ item.data.name }}</span>
                  <span class="match-count-tag">{{ item.data.matchCount }} 筆匹配</span>
                </template>

                <!-- 敏感字元匹配行 -->
                <template v-else>
                  <span class="line-badge">行 {{ item.data.scanResult?.line_num }}</span>
                  <span class="pattern-badge">{{ item.data.scanResult?.pattern_name }}</span>
                  <span class="matched-snippet" :title="item.data.scanResult?.matched_text">
                    {{ item.data.scanResult?.matched_text }}
                  </span>
                </template>
              </span>
            </div>
          </template>
        </div>
      </div>

      <!-- 空白狀態 HUD -->
      <div
        v-if="results.length === 0 && !store.isScanning"
        class="empty-state"
      >
        <div class="empty-icon">📁</div>
        <p>尚未開始掃描或未發現任何敏感個資檔案</p>
      </div>
    </div>
  </main>
</template>

<style scoped>
.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #fff;
}

.content-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.header-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-info h1 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
  color: #2c3e50;
}

.count-badge {
  background: #f1f3f5;
  color: #495057;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 0.8rem;
  font-weight: 700;
}

/* 頂部操作列 */
.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* 分段檢視切換控制器 */
.view-toggle {
  display: flex;
  background: #f1f3f5;
  padding: 3px;
  border-radius: 8px;
}

.toggle-btn {
  background: transparent;
  border: none;
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  color: #495057;
  cursor: pointer;
  transition: all 0.15s ease;
  display: flex;
  align-items: center;
  gap: 4px;
}

.toggle-btn:hover {
  color: #212529;
}

.toggle-btn.active {
  background: #fff;
  color: #007bff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

/* 樹狀輔助按鈕 */
.tree-controls {
  display: flex;
  gap: 6px;
}

.control-btn {
  background: #fff;
  border: 1px solid #dee2e6;
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 0.78rem;
  font-weight: 600;
  color: #495057;
  cursor: pointer;
  transition: all 0.15s ease;
}

.control-btn:hover {
  background: #f8f9fa;
  border-color: #ced4da;
  color: #212529;
}

/* 表格標頭 */
.table-header {
  display: flex;
  background: #fafafc;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  padding: 0 24px;
  font-weight: 600;
  font-size: 0.8rem;
  color: #7f8c8d;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  height: 38px;
  align-items: center;
}

.table-header.is-tree {
  font-size: 0.82rem;
  letter-spacing: 0.2px;
}

.tree-header {
  flex: 1;
  padding-left: 12px;
}

/* 表格內容與滾動 */
.table-container {
  flex: 1;
  overflow: auto;
}

.table-row {
  display: flex;
  align-items: center;
  padding: 0 24px;
  border-bottom: 1px solid #f8f9fa;
  font-size: 0.88rem;
  transition: background 0.15s ease-in-out;
  box-sizing: border-box;
}

.table-row:hover {
  background: #fafafc;
}

.col {
  padding: 10px 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  box-sizing: border-box;
}

/* 列表檢視欄位樣式 */
.type {
  width: 130px;
}

.path {
  flex: 1;
  color: #007bff;
  font-weight: 500;
  cursor: pointer;
}

.path:hover {
  text-decoration: underline;
  color: #0056b3;
}

.line {
  width: 80px;
  color: #6c757d;
  font-family: monospace;
}

.matched {
  width: 260px;
  color: #dc3545;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.82rem;
  font-weight: 600;
}

.badge {
  background: rgba(0, 123, 255, 0.08);
  color: #007bff;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
  display: inline-block;
}

/* 樹狀結構檢視專用樣式 */
.tree-node-content {
  display: flex;
  align-items: center;
  width: 100%;
  height: 100%;
  cursor: pointer;
  user-select: none;
  box-sizing: border-box;
}

/* 資料夾與檔案節點樣式 */
.node-folder {
  font-weight: 600;
}

.node-match {
  background: rgba(220, 53, 69, 0.01);
}

.node-match:hover {
  background: rgba(220, 53, 69, 0.03) !important;
}

/* 旋轉指示圖示 */
.chevron-icon {
  font-size: 0.62rem;
  color: #adb5bd;
  margin-right: 6px;
  width: 12px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.18s ease;
  transform: rotate(-90deg); /* 預設收合朝右 */
}

.chevron-icon.is-open {
  transform: rotate(0deg); /* 展開時朝下 */
}

.chevron-placeholder {
  width: 18px;
}

.node-icon {
  font-size: 1rem;
  margin-right: 8px;
  display: inline-flex;
  align-items: center;
}

.node-name {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.name-text {
  font-weight: 600;
  color: #343a40;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-file .name-text {
  color: #495057;
  font-weight: 500;
}

.match-count-tag {
  background: rgba(0, 123, 255, 0.08);
  color: #007bff;
  padding: 2px 7px;
  border-radius: 10px;
  font-size: 0.7rem;
  font-weight: 700;
  flex-shrink: 0;
}

/* 匹配項樣式微調 */
.is-match-text {
  font-family: system-ui, sans-serif;
  width: 100%;
}

.line-badge {
  background: #f1f3f5;
  color: #495057;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.72rem;
  font-family: monospace;
  font-weight: 700;
  flex-shrink: 0;
}

.pattern-badge {
  background: rgba(220, 53, 69, 0.08);
  color: #dc3545;
  padding: 2px 7px;
  border-radius: 4px;
  font-size: 0.72rem;
  font-weight: 700;
  flex-shrink: 0;
}

.matched-snippet {
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.8rem;
  color: #212529;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  font-weight: 500;
}

/* 空白狀態 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #95a5a6;
  gap: 16px;
  padding: 40px;
  text-align: center;
}

.empty-icon {
  font-size: 3rem;
}

.empty-state p {
  font-size: 0.95rem;
  font-weight: 500;
  margin: 0;
}
</style>
