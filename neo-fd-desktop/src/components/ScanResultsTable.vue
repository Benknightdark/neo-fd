<script setup lang="ts">
import { useVirtualList } from '@vueuse/core';
import { storeToRefs } from 'pinia';
import { computed, ref, watch } from 'vue';
import type { ScanResult } from '../api/ipc';
import { useScanStore } from '../stores/scan';

const store = useScanStore();
const { results } = storeToRefs(store);

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

// 樹狀節點介面定義
interface TreeNode {
  id: string;
  name: string;
  type: 'folder' | 'file' | 'match';
  depth: number;
  isOpen: boolean;
  parentId: string | null;
  children: string[];
  matchCount: number;
  scanResult?: ScanResult;
}

// 記錄被折疊的節點 ID
const collapsedIds = ref<Set<string>>(new Set());

// 一鍵展開全部
function expandAll() {
  collapsedIds.value = new Set();
}

// 一鍵折疊全部
function collapseAll() {
  const allIds = new Set<string>();
  flattenedNodes.value.forEach((node) => {
    if (node.type !== 'match') {
      allIds.add(node.id);
    }
  });
  collapsedIds.value = allIds;
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
}

// 共同前綴路徑擷取 Helper 函數，避免深度過深的無意義空殼節點
function getCommonDirectory(paths: string[]): string {
  if (paths.length === 0) return '';
  if (paths.length === 1) {
    const parts = paths[0].split('/');
    parts.pop();
    return parts.join('/');
  }
  const splitPaths = paths.map((p) => p.split('/'));
  const common: string[] = [];
  const firstPathParts = splitPaths[0];

  for (let i = 0; i < firstPathParts.length - 1; i++) {
    const part = firstPathParts[i];
    const isCommon = splitPaths.every((parts) => parts[i] === part);
    if (isCommon) {
      common.push(part);
    } else {
      break;
    }
  }
  return common.join('/');
}

// 建構完整的反應式多級樹狀結構，並根據 collapsedIds 計算扁平化數組
const flattenedNodes = computed<TreeNode[]>(() => {
  if (results.value.length === 0) return [];

  const paths = results.value.map((r) => r.path.replace(/\\/g, '/'));
  const commonPrefix = getCommonDirectory(paths);

  const nodesMap = new Map<string, TreeNode>();
  const root: TreeNode = {
    id: 'root',
    name: 'Root',
    type: 'folder',
    depth: -1,
    isOpen: true,
    parentId: null,
    children: [],
    matchCount: 0,
  };
  nodesMap.set('root', root);

  results.value.forEach((res, index) => {
    const normalizedPath = res.path.replace(/\\/g, '/');
    let relativePath = normalizedPath;
    if (commonPrefix && normalizedPath.startsWith(commonPrefix)) {
      relativePath = normalizedPath.slice(commonPrefix.length);
      if (relativePath.startsWith('/')) {
        relativePath = relativePath.slice(1);
      }
    }

    const pathParts = relativePath.split('/');
    let currentParentId = 'root';
    let accumulatedPath = commonPrefix;

    pathParts.forEach((part, partIndex) => {
      const isLast = partIndex === pathParts.length - 1;
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${part}` : part;
      const nodeId = accumulatedPath;
      const depth = partIndex;

      if (!nodesMap.has(nodeId)) {
        const parentNode = nodesMap.get(currentParentId);
        if (parentNode) {
          parentNode.children.push(nodeId);
        }

        const nodeType = isLast ? 'file' : 'folder';
        const isOpen = !collapsedIds.value.has(nodeId);

        const newNode: TreeNode = {
          id: nodeId,
          name: part,
          type: nodeType,
          depth: depth,
          isOpen: isOpen,
          parentId: currentParentId,
          children: [],
          matchCount: 0,
        };
        nodesMap.set(nodeId, newNode);
      }

      const node = nodesMap.get(nodeId);
      if (node) {
        node.matchCount += 1;
      }
      currentParentId = nodeId;
    });

    // 建立匹配行節點
    const fileNode = nodesMap.get(currentParentId);
    if (fileNode) {
      const matchNodeId = `${res.path}:${res.line_num}:${res.pattern_name}:${index}`;
      const isOpen = !collapsedIds.value.has(matchNodeId);

      const matchNode: TreeNode = {
        id: matchNodeId,
        name: `行 ${res.line_num}: [${res.pattern_name}] ${res.matched_text}`,
        type: 'match',
        depth: fileNode.depth + 1,
        isOpen: isOpen,
        parentId: currentParentId,
        children: [],
        matchCount: 1,
        scanResult: res,
      };

      nodesMap.set(matchNodeId, matchNode);
      fileNode.children.push(matchNodeId);
    }

    const rootNode = nodesMap.get('root');
    if (rootNode) {
      rootNode.matchCount += 1;
    }
  });

  // 深度優先 (DFS) 遍歷以打平樹狀結構
  const flattened: TreeNode[] = [];
  function traverse(nodeId: string) {
    const node = nodesMap.get(nodeId);
    if (!node) return;

    if (nodeId !== 'root') {
      flattened.push(node);
    }

    if (node.isOpen) {
      const childNodes = node.children
        .map((id) => nodesMap.get(id))
        .filter((n): n is TreeNode => !!n);

      // 精緻排序：資料夾排在檔案前面，接著是檔案，按名稱排序
      childNodes.sort((a, b) => {
        if (a.type !== b.type) {
          if (a.type === 'folder') return -1;
          if (b.type === 'folder') return 1;
          if (a.type === 'file') return -1;
          if (b.type === 'file') return 1;
        }
        return a.name.localeCompare(b.name);
      });

      for (const child of childNodes) {
        traverse(child.id);
      }
    }
  }

  traverse('root');
  return flattened;
});

// 統合成單一虛擬滾動列表資料來源，依檢視模式切換
type DisplayItem =
  | { type: 'list-item'; data: ScanResult; index: number }
  | { type: 'tree-item'; data: TreeNode; index: number };

const displayList = computed<DisplayItem[]>(() => {
  if (viewMode.value === 'list') {
    return results.value.map((r, i) => ({
      type: 'list-item' as const,
      data: r,
      index: i,
    }));
  }
  return flattenedNodes.value.map((n, i) => ({
    type: 'tree-item' as const,
    data: n,
    index: i,
  }));
});

// 註冊高性能虛擬滾動列表
const { list, containerProps, wrapperProps } = useVirtualList(displayList, {
  itemHeight: 45,
});

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
          :key="item.index"
          class="table-row"
          :class="[
            item.data.type === 'list-item' ? 'list-row' : 'tree-row',
            item.data.type === 'tree-item' ? `node-${item.data.data.type}` : ''
          ]"
          :style="{ height: '45px' }"
        >
          <!-- A. 列表檢視渲染 -->
          <template v-if="item.data.type === 'list-item'">
            <div class="col type">
              <span class="badge">{{ item.data.data.pattern_name }}</span>
            </div>
            <div
              class="col path"
              :title="item.data.data.path"
              @click="emit('open-drawer', item.data.data)"
            >
              {{ item.data.data.path }}
            </div>
            <div class="col line">{{ item.data.data.line_num }}</div>
            <div class="col matched">{{ item.data.data.matched_text }}</div>
          </template>

          <!-- B. 樹狀檢視渲染 (Multi-stage Folder/File/Match nodes) -->
          <template v-else>
            <div
              class="tree-node-content"
              :style="{ paddingLeft: `${item.data.data.depth * 20 + 20}px` }"
              @click="handleNodeClick(item.data.data)"
            >
              <!-- 展開/收合狀態旋轉箭頭 -->
              <span
                v-if="item.data.data.type !== 'match'"
                class="chevron-icon"
                :class="{ 'is-open': item.data.data.isOpen }"
              >
                ▼
              </span>
              <span v-else class="chevron-placeholder"></span>

              <!-- 語意化圖示 -->
              <span class="node-icon">
                <template v-if="item.data.data.type === 'folder'">
                  {{ item.data.data.isOpen ? '📂' : '📁' }}
                </template>
                <template v-else-if="item.data.data.type === 'file'">
                  📄
                </template>
                <template v-else>
                  🔍
                </template>
              </span>

              <!-- 內容顯示 -->
              <span class="node-name" :class="{ 'is-match-text': item.data.data.type === 'match' }">
                <!-- 資料夾或檔案 -->
                <template v-if="item.data.data.type !== 'match'">
                  <span class="name-text" :title="item.data.data.name">{{ item.data.data.name }}</span>
                  <span class="match-count-tag">{{ item.data.data.matchCount }} 筆匹配</span>
                </template>

                <!-- 敏感字元匹配行 -->
                <template v-else>
                  <span class="line-badge">行 {{ item.data.data.scanResult?.line_num }}</span>
                  <span class="pattern-badge">{{ item.data.data.scanResult?.pattern_name }}</span>
                  <span class="matched-snippet" :title="item.data.data.scanResult?.matched_text">
                    {{ item.data.data.scanResult?.matched_text }}
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

