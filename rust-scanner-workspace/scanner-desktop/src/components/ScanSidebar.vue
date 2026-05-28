<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useScanStore } from '../stores/scan';

const store = useScanStore();

// 自定義正規表達式的語法錯誤訊息
const regexError = ref('');

// 監聽並即時校驗使用者自定義正規表達式 (Regex) 的語法正確性
watch(
  () => store.customPattern,
  (val) => {
    if (!val) {
      regexError.value = '';
      return;
    }
    try {
      new RegExp(val);
      regexError.value = '';
    } catch (err) {
      regexError.value = '正規表達式語法錯誤';
    }
  },
);

// 計算屬性守衛：確保至少選擇一個內建模式，或輸入了語法正確的自定義模式才可進行掃描
const canScan = computed(() => {
  if (store.isScanning) return false;
  if (regexError.value) return false;

  const hasBuiltin = store.selectedPatterns.some((p) => p.enabled);
  const hasCustom = !!store.customPattern;

  return hasBuiltin || hasCustom;
});

// 觸發掃描程序
function handleScan() {
  if (!canScan.value) return;
  store.startScan();
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>掃描設定</h2>
    </div>

    <div class="field">
      <label for="scan-path">目標路徑</label>
      <input
        id="scan-path"
        v-model="store.scanPath"
        type="text"
        placeholder="例如: /Users/name/Documents"
        :disabled="store.isScanning"
      />
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
          :class="{ 'input-error': regexError }"
        />
        <Transition name="fade">
          <span v-if="regexError" class="error-msg" role="alert">
            ⚠️ {{ regexError }}
          </span>
        </Transition>
      </div>
    </div>

    <button
      class="scan-btn"
      @click="handleScan"
      :disabled="!canScan"
    >
      <span v-if="store.isScanning" class="spinner"></span>
      {{ store.isScanning ? '掃描中...' : '開始掃描' }}
    </button>
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

input[type="text"] {
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

input[type="text"]:focus {
  border-color: #007bff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.15);
  outline: none;
}

input[type="text"]:disabled {
  background: #f8f9fa;
  color: #adb5bd;
  cursor: not-allowed;
}

.input-error {
  border-color: hsl(350, 80%, 55%) !important;
}

.input-error:focus {
  box-shadow: 0 0 0 3px rgba(220, 53, 69, 0.15) !important;
}

.error-msg {
  color: hsl(350, 80%, 45%);
  font-size: 0.75rem;
  font-weight: 500;
  margin-top: 4px;
  display: block;
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

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  border-top-color: #white;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
</style>
