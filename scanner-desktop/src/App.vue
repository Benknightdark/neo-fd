<script setup lang="ts">
// 引入 Vue 生命週期 hooks 與響應式 API
import { onMounted, onUnmounted, ref } from 'vue';
import type { ScanResult } from './api/ipc';

// 引入子組件模組
import CodeViewerDrawer from './components/CodeViewerDrawer.vue';
import ScanResultsTable from './components/ScanResultsTable.vue';
import ScanSidebar from './components/ScanSidebar.vue';
import ToastNotifications from './components/ToastNotifications.vue';

// 引入全域掃描 Store
import { useScanStore } from './stores/scan';

const store = useScanStore();

// 原始碼檢視抽屜控制狀態
const isDrawerOpen = ref(false);
const activeResult = ref<ScanResult | null>(null);

// 開啟原始碼檢視抽屜
function handleOpenDrawer(result: ScanResult) {
  activeResult.value = result;
  isDrawerOpen.value = true;
}

// 關閉原始碼檢視抽屜
function handleCloseDrawer() {
  isDrawerOpen.value = false;
}

// 元件掛載時註冊 Tauri 後端事件監聽器
onMounted(() => {
  store.init();
});

// 元件銷毀時卸載監聽器，防止記憶體洩漏
onUnmounted(() => {
  store.cleanup();
});
</script>

<template>
  <div class="app-layout">
    <!-- Main configuration settings sidebar -->
    <ScanSidebar />

    <!-- Virtual scrolling results viewport table -->
    <ScanResultsTable @open-drawer="handleOpenDrawer" />

    <!-- Sliding preview details panel for raw source viewer -->
    <CodeViewerDrawer
      :is-open="isDrawerOpen"
      :active-result="activeResult"
      @close="handleCloseDrawer"
    />

    <!-- Premium glassmorphic notifications hud -->
    <ToastNotifications />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  background: #ffffff;
}
</style>
