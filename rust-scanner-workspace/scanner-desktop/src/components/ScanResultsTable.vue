<script setup lang="ts">
import { useVirtualList } from '@vueuse/core';
import type { ScanResult } from '../api/ipc';
import { useScanStore } from '../stores/scan';

const store = useScanStore();

// 宣告開口事件：當使用者點擊結果檔案時，發送該洩漏項目給父元件以開啟抽屜
const emit = defineEmits<(e: 'open-drawer', result: ScanResult) => void>();

// 註冊高性能虛擬滾動列表 (Virtual Scroll)，在超大型資料下能大幅降低 DOM 節點渲染開銷
const { list, containerProps, wrapperProps } = useVirtualList(store.results, {
  itemHeight: 45, // 固定單行高度以供虛擬滾動計算
});
</script>

<template>
  <main class="content">
    <header class="content-header">
      <div class="header-info">
        <h1>掃描結果</h1>
        <span class="count-badge">{{ store.results.length }}</span>
      </div>
    </header>

    <div class="table-header">
      <div class="col type">類別</div>
      <div class="col path">檔案路徑</div>
      <div class="col line">行號</div>
      <div class="col matched">匹配內容</div>
    </div>

    <div v-bind="containerProps" class="table-container">
      <div v-bind="wrapperProps">
        <div
          v-for="item in list"
          :key="item.index"
          class="table-row"
          :style="{ height: '45px' }"
        >
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
        </div>
      </div>
      
      <div
        v-if="store.results.length === 0 && !store.isScanning"
        class="empty-state"
      >
        <div class="empty-icon">📁</div>
        <p>尚未開始掃描或未發現任何個資敏感檔案</p>
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
  padding: 20px 24px;
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
}

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
