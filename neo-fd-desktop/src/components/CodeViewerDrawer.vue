<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { computed, nextTick, ref, watch } from 'vue';
import { type ScanResult, scannerApi, systemApi } from '../api/ipc';
import { useNotificationStore } from '../stores/notification';
import { useScanStore } from '../stores/scan';

// 定義傳入屬性：控制抽屜開啟/關閉，以及目前選中的洩漏點資料
const props = defineProps<{
  isOpen: boolean;
  activeResult: ScanResult | null;
}>();

// 宣告關閉事件，以回報父元件進行狀態清理
const emit = defineEmits<(e: 'close') => void>();

const store = useScanStore();
const { results } = storeToRefs(store);
const notificationStore = useNotificationStore();

// 本地元件渲染狀態
const isLoadingFile = ref(false);
const drawerPath = ref('');
const drawerTargetLine = ref<number | null>(null);
const drawerMatchedText = ref('');
const drawerActiveMatchIndex = ref(0);
const drawerLines = ref<string[]>([]);
const drawerError = ref('');
const codeViewerRef = ref<HTMLElement | null>(null);

// 新增：編輯模式狀態與內容緩衝
const isEditing = ref(false);
const editedContent = ref('');
const isSaving = ref(false);

// 計算屬性：篩選出當前開啟檔案的所有個資洩漏點，並依據行號遞增排序
const activeFileMatches = computed(() => {
  if (!drawerPath.value) return [];
  return [...results.value]
    .filter((r) => r.path === drawerPath.value)
    .sort((a, b) => a.line_num - b.line_num);
});

// 非同步加載目標檔案內容，並解析出跳轉 HUD 指標
async function loadFileContent(result: ScanResult) {
  isLoadingFile.value = true;
  drawerPath.value = result.path;
  drawerTargetLine.value = result.line_num;
  drawerMatchedText.value = result.matched_text;
  drawerLines.value = [];
  drawerError.value = '';
  isEditing.value = false; // 切換檔案時自動離開編輯模式

  // 尋找目前選取點在該檔案所有洩漏點中的索引序號
  const clickedIndex = activeFileMatches.value.findIndex(
    (r) =>
      r.line_num === result.line_num && r.matched_text === result.matched_text,
  );
  drawerActiveMatchIndex.value = clickedIndex !== -1 ? clickedIndex : 0;

  try {
    // 呼叫 IPC 讀取本地檔案內容
    const content = await scannerApi.readFileContent(result.path);
    drawerLines.value = content.split(/\r?\n/);

    // 平滑捲動至目標行號，並將其垂直置中於檢視窗
    await nextTick();
    scrollToActiveLine();
  } catch (err) {
    drawerError.value = (err as Error).message || '無法載入檔案內容';
  } finally {
    isLoadingFile.value = false;
  }
}

// 平滑滾動視窗定位，將金色高亮行置於容器正中央
function scrollToActiveLine() {
  if (isEditing.value) return; // 編輯中無須捲動
  const targetElement = codeViewerRef.value?.querySelector('.highlight-line');
  if (targetElement) {
    targetElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }
}

// 監聽抽屜開啟狀態與目前結果：支援首次開啟、已開啟時切換結果，以及關閉後延遲清理
watch(
  () => [props.isOpen, props.activeResult] as const,
  async ([isOpen, result], [wasOpen, previousResult]) => {
    if (isOpen && result && (!wasOpen || result !== previousResult)) {
      await loadFileContent(result);
      return;
    }

    if (!isOpen) {
      setTimeout(() => {
        if (!props.isOpen) {
          drawerLines.value = [];
          drawerPath.value = '';
          drawerTargetLine.value = null;
          drawerMatchedText.value = '';
          drawerActiveMatchIndex.value = 0;
          drawerError.value = '';
          isEditing.value = false;
          editedContent.value = '';
        }
      }, 400);
    }
  },
);

// 行分割資料型態介面
interface TextPart {
  text: string;
  isMatch: boolean;
}

// 安全分割行文字，標記出 Regex 匹配的敏感文字片段以供高亮渲染
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

// HUD 指標導航跳轉（前一個 / 後一個洩漏點）
async function jumpToMatch(index: number) {
  if (isEditing.value) return; // 編輯中禁用跳轉

  const targetResult = activeFileMatches.value[index];
  if (!targetResult) return;

  drawerActiveMatchIndex.value = index;
  drawerTargetLine.value = targetResult.line_num;
  drawerMatchedText.value = targetResult.matched_text;

  // 觸發視窗平滑置中滾動
  await nextTick();
  scrollToActiveLine();
}

// 自檔案路徑截取純檔案名稱
function getFileName(fullPath: string): string {
  const parts = fullPath.split(/[/\\]/);
  return parts[parts.length - 1] || fullPath;
}

// 拖拉抽屜寬度狀態
const drawerWidth = ref(800); // 預設寬度 800px
const isResizing = ref(false);

function startResize(e: MouseEvent) {
  isResizing.value = true;
  const startX = e.clientX;
  const startWidth = drawerWidth.value;

  // 為了防止拖動時選取到頁面文字，並設定全域游標
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'ew-resize';

  function doResize(moveEvent: MouseEvent) {
    if (!isResizing.value) return;
    const deltaX = startX - moveEvent.clientX; // 面板在右邊，滑鼠往左移動（clientX 變小）時 deltaX 為正，寬度應增加
    const newWidth = startWidth + deltaX;

    // 寬度界限約束
    const minWidth = 450;
    const maxWidth = window.innerWidth * 0.95;
    drawerWidth.value = Math.max(minWidth, Math.min(maxWidth, newWidth));
  }

  function stopResize() {
    isResizing.value = false;
    document.body.style.userSelect = '';
    document.body.style.cursor = '';
    window.removeEventListener('mousemove', doResize);
    window.removeEventListener('mouseup', stopResize);
  }

  window.addEventListener('mousemove', doResize);
  window.addEventListener('mouseup', stopResize);
}

// 新增：綁定編輯器 DOM 控制
const lineNumbersRef = ref<HTMLElement | null>(null);
const textareaRef = ref<HTMLElement | null>(null);

// 新增：即時計算編輯狀態下的總行數
const editedLinesCount = computed(() => {
  return editedContent.value.split(/\r?\n/).length || 1;
});

// 新增：同步滾動編輯器行號與輸入區
function syncScroll() {
  if (textareaRef.value && lineNumbersRef.value) {
    lineNumbersRef.value.scrollTop = textareaRef.value.scrollTop;
  }
}

// 新增：精準定位並反白選取編輯器中的個資敏感文字
function focusAndSelectLeak() {
  if (!textareaRef.value || !props.activeResult) return;

  const lines = drawerLines.value;
  const targetLineIdx = props.activeResult.line_num - 1;
  const matchedText = props.activeResult.matched_text;

  if (targetLineIdx < 0 || targetLineIdx >= lines.length) return;

  // 計算目標行之前的字元偏偏移長度
  let charOffset = 0;
  for (let i = 0; i < targetLineIdx; i++) {
    charOffset += lines[i].length + 1; // +1 代表換行符
  }

  const targetLineText = lines[targetLineIdx];
  const matchInLineIdx = targetLineText.indexOf(matchedText);

  if (matchInLineIdx !== -1) {
    const selectionStart = charOffset + matchInLineIdx;
    const selectionEnd = selectionStart + matchedText.length;

    nextTick(() => {
      if (textareaRef.value) {
        textareaRef.value.focus();
        textareaRef.value.setSelectionRange(selectionStart, selectionEnd);

        // 滾動到目標行並置中
        const lineHeight = 22; // 行高 22px
        const scrollOffset = Math.max(0, targetLineIdx * lineHeight - 150);
        textareaRef.value.scrollTop = scrollOffset;
        if (lineNumbersRef.value) {
          lineNumbersRef.value.scrollTop = scrollOffset;
        }
      }
    });
  }
}

// 新增：切換至編輯模式
function startEdit() {
  editedContent.value = drawerLines.value.join('\n');
  isEditing.value = true;

  // 100ms 延遲以確保 DOM 已經妥善渲染，隨後自動定位反白
  setTimeout(() => {
    focusAndSelectLeak();
  }, 100);
}

// 新增：取消編輯並回復
function cancelEdit() {
  isEditing.value = false;
  editedContent.value = '';
}

// 新增：儲存編輯內容
async function saveEdit() {
  if (isSaving.value) return;
  isSaving.value = true;

  try {
    await scannerApi.writeFileContent(drawerPath.value, editedContent.value);
    notificationStore.add(
      '檔案修改已儲存。建議您重新掃描以驗證敏感個資是否已成功遮蔽！',
      'success',
    );
    isEditing.value = false;

    // 重新載入修改後的內容
    if (props.activeResult) {
      await loadFileContent(props.activeResult);
    }
  } catch (err) {
    notificationStore.add(`儲存修改失敗: ${(err as Error).message}`, 'error');
  } finally {
    isSaving.value = false;
  }
}

// 新增：二次確認並永久刪除檔案
async function confirmDelete() {
  const file = getFileName(drawerPath.value);
  const isConfirmed = confirm(
    `⚠️ 警告：您確定要永久刪除此檔案嗎？\n\n檔案名稱: ${file}\n完整路徑: ${drawerPath.value}\n\n此操作會將該檔案自硬碟中徹底移除且「無法復原」！請確認這不是系統關鍵檔案。`,
  );

  if (!isConfirmed) return;

  try {
    await scannerApi.deleteFile(drawerPath.value);
    notificationStore.add('檔案已成功永久刪除。', 'success');

    // 即時將其從前端 Scan results store 中濾除，UI 目錄樹即時反應消失
    store.removeResultsByPath(drawerPath.value);

    // 關閉抽屜
    emit('close');
  } catch (err) {
    notificationStore.add(`刪除檔案失敗: ${(err as Error).message}`, 'error');
  }
}
</script>

<template>
  <div class="drawer-container" :class="{ active: isOpen }">
    <div class="drawer-backdrop" @click="isSaving ? null : emit('close')"></div>
    <aside class="drawer-panel" :style="{ width: drawerWidth + 'px' }" :class="{ resizing: isResizing }">
      <div class="drawer-resizer" @mousedown="startResize"></div>
      <header class="drawer-header">
        <div class="drawer-title-group">
          <h2>{{ getFileName(drawerPath) }}</h2>
          <span class="drawer-path" :title="drawerPath">{{ drawerPath }}</span>
        </div>
        <div class="drawer-actions">
          <!-- 唯讀模式下的功能按鈕 -->
          <template v-if="!isEditing && !drawerError && !isLoadingFile">
            <button
              class="edit-btn"
              title="修改檔案敏感內容"
              @click="startEdit"
            >
              ✏️ 編輯內容
            </button>
            <button
              v-if="drawerPath"
              class="delete-btn"
              title="自硬碟永久刪除此檔案"
              @click="confirmDelete"
            >
              🗑️ 刪除檔案
            </button>
            <button
              class="open-system-btn"
              title="使用系統預設應用程式開啟"
              @click="systemApi.openFile(drawerPath)"
            >
              🗁 外部開啟
            </button>
          </template>

          <!-- 編輯模式下的操作按鈕 -->
          <template v-else-if="isEditing">
            <button
              class="save-btn"
              :disabled="isSaving"
              @click="saveEdit"
              title="儲存檔案修改"
            >
              {{ isSaving ? '💾 儲存中...' : '💾 儲存修改' }}
            </button>
            <button
              class="cancel-btn"
              :disabled="isSaving"
              @click="cancelEdit"
              title="放棄修改並取消"
            >
              ❌ 取消
            </button>
          </template>

          <button class="close-btn" :disabled="isSaving" @click="emit('close')" aria-label="關閉面板">
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

        <!-- Code Viewer & Editor Content -->
        <div v-else class="code-viewer-container">
          <!-- 敏感字元高亮對照 HUD (僅在編輯模式下且有 activeResult 時呈現) -->
          <div v-if="isEditing && activeResult" class="editor-leak-hud">
            <span class="warning-flashing-icon">💡</span>
            <span class="hud-text">
              請在下方修正第 <strong>{{ activeResult.line_num }}</strong> 行的個資洩漏文字：
              <code class="leak-highlight-token" :title="activeResult.matched_text">
                {{ activeResult.matched_text }}
              </code>
            </span>
            <button class="locate-btn" @click="focusAndSelectLeak" title="一鍵聚焦並反白個資文字">
              🎯 重新定位選取
            </button>
          </div>

          <div class="code-header-info">
            <div class="code-stats">
              <span>高亮行號: <strong>{{ isEditing ? '編輯中' : drawerTargetLine }}</strong></span>
              <span class="divider">|</span>
              <span>共 <strong>{{ drawerLines.length }}</strong> 行</span>
            </div>
            <div v-if="!isEditing && activeFileMatches.length > 1" class="navigator-group">
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

          <!-- A. 唯讀狀態之語法高亮 Code Viewer -->
          <div v-if="!isEditing" ref="codeViewerRef" class="code-viewer">
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

          <!-- B. 編輯狀態之自定義極簡 TextArea 編輯器 (帶有同步滾動行號列) -->
          <div v-else class="code-editor-wrapper">
            <!-- 編輯器左側行號列 -->
            <div ref="lineNumbersRef" class="editor-line-numbers">
              <div
                v-for="n in editedLinesCount"
                :key="n"
                class="editor-line-number"
                :class="{ 'highlight-editor-line': n === drawerTargetLine }"
              >
                {{ n }}
              </div>
            </div>
            <!-- 編輯器右側文字輸入區 -->
            <textarea
              ref="textareaRef"
              class="code-editor-textarea"
              v-model="editedContent"
              :disabled="isSaving"
              @scroll="syncScroll"
              placeholder="請在此處編輯或遮蔽敏感內容，點擊右上角「儲存修改」寫入磁碟..."
              spellcheck="false"
            ></textarea>
          </div>
        </div>
      </div>
    </aside>
  </div>
</template>

<style scoped>
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
  /* width 由 inline style 動態控制 */
  max-width: 90vw;
  height: 100%;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: -10px 0 30px rgba(0, 0, 0, 0.12);
  border-left: 1px solid rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
  transform: translateX(100%);
  transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1), width 0.1s ease-out; /* 寬度微調時帶點平滑感，但為防拖曳延遲，時間設極短 */
}

/* 拖動時關閉 transition，使滑鼠拖拉極致流暢 */
.drawer-panel.resizing {
  transition: none;
}

/* Resizer bar style for dragging width */
.drawer-resizer {
  position: absolute;
  top: 0;
  left: 0;
  width: 6px;
  height: 100%;
  cursor: ew-resize;
  z-index: 10;
  transition: background-color 0.2s ease, box-shadow 0.2s ease;
}

.drawer-resizer:hover,
.drawer-panel.resizing .drawer-resizer {
  background-color: rgba(0, 123, 255, 0.45); /* Premium blue glow */
  box-shadow: 0 0 10px rgba(0, 123, 255, 0.6);
}

.drawer-container.active .drawer-panel {
  transform: translateX(0);
}

/* Drawer Header Styles */
.drawer-header {
  padding: 18px 24px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
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
  font-size: 1.2rem;
  font-weight: 700;
  color: #2c3e50;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drawer-path {
  font-size: 0.78rem;
  color: #7f8c8d;
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
  padding: 8px 14px;
  background: #e9ecef;
  color: #495057;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 0.82rem;
  cursor: pointer;
  font-weight: 600;
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
  font-size: 1.8rem;
  color: #bdc3c7;
  cursor: pointer;
  line-height: 1;
  padding: 4px;
  border-radius: 50%;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.close-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #34495e;
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
  color: #7f8c8d;
  gap: 16px;
}

.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid rgba(0, 123, 255, 0.1);
  border-radius: 50%;
  border-top-color: #007bff;
  animation: spin 0.8s ease-in-out infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon {
  font-size: 2.5rem;
}

.drawer-error p {
  color: #dc3545;
  font-weight: 600;
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
  padding: 10px 24px;
  background: #f1f3f5;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  font-size: 0.8rem;
  color: #495057;
  display: flex;
  justify-content: space-between;
  align-items: center;
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
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 20px;
  padding: 3px 12px;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.03);
}

.nav-btn {
  background: none;
  border: none;
  font-size: 0.72rem;
  color: #495057;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 10px;
  transition: all 0.2s ease;
  font-weight: 700;
}

.nav-btn:hover:not(:disabled) {
  background: rgba(0, 0, 0, 0.05);
  color: #007bff;
}

.nav-btn:disabled {
  color: #dee2e6;
  cursor: not-allowed;
}

.match-indicator {
  font-size: 0.72rem;
  color: #7f8c8d;
  font-weight: 600;
}

.code-viewer {
  flex: 1;
  overflow: auto;
  background: #23241f; /* Premium Monokai dark code theme */
  color: #f8f8f2;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.82rem;
  padding: 12px 0;
}

.code-line {
  display: flex;
  min-height: 22px;
  line-height: 22px;
  width: 100%;
  white-space: pre;
}

.line-number {
  width: 55px;
  min-width: 55px;
  text-align: right;
  padding-right: 18px;
  color: #75715e;
  user-select: none;
  background: rgba(0, 0, 0, 0.15);
  border-right: 1px solid rgba(255, 255, 255, 0.05);
}

.line-content {
  margin: 0;
  padding-left: 18px;
  flex: 1;
  overflow-x: auto;
  color: #f8f8f2;
}

/* Golden Highlight style for the matched leak point line */
.code-line.highlight-line {
  background: rgba(255, 193, 7, 0.12); /* Golden tint background */
  position: relative;
}

.code-line.highlight-line::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 4px;
  background: #f1c40f; /* Gold boundary highlight bar */
  box-shadow: 0 0 8px #f1c40f;
}

.code-line.highlight-line .line-number {
  color: #f1c40f;
  background: rgba(255, 193, 7, 0.08);
  font-weight: 700;
}

.code-line.highlight-line .line-content {
  color: #ffffff;
  font-weight: 600;
}

/* Ruby red glow style for specific matched regex token segments */
.match-segment {
  background: rgba(220, 53, 69, 0.35); /* Ruby red transparent background */
  color: #ff4757; /* Bright soft red text */
  border: 1px solid rgba(220, 53, 69, 0.5);
  border-radius: 4px;
  padding: 1px 4px;
  font-weight: 700;
  box-shadow: 0 0 6px rgba(220, 53, 69, 0.2);
}

/* 編輯修改按鈕 */
.edit-btn {
  padding: 8px 14px;
  background: #f1f3f5;
  color: #495057;
  border: 1px solid #dee2e6;
  border-radius: 6px;
  font-size: 0.82rem;
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.edit-btn:hover {
  background: #e9ecef;
  border-color: #ced4da;
  color: #007bff;
}

/* 刪除按鈕 (紅色 glassmorphic) */
.delete-btn {
  padding: 8px 14px;
  background: rgba(220, 53, 69, 0.08);
  color: #dc3545;
  border: 1px solid rgba(220, 53, 69, 0.25);
  border-radius: 6px;
  font-size: 0.82rem;
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.delete-btn:hover {
  background: #dc3545;
  border-color: #dc3545;
  color: #fff;
  box-shadow: 0 2px 6px rgba(220, 53, 69, 0.2);
}

/* 儲存按鈕 (實心藍) */
.save-btn {
  padding: 8px 14px;
  background: #007bff;
  color: #fff;
  border: 1px solid #007bff;
  border-radius: 6px;
  font-size: 0.82rem;
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.save-btn:hover:not(:disabled) {
  background: #0056b3;
  border-color: #0056b3;
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 取消按鈕 (灰) */
.cancel-btn {
  padding: 8px 14px;
  background: #e9ecef;
  color: #495057;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 0.82rem;
  cursor: pointer;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.2s;
}

.cancel-btn:hover:not(:disabled) {
  background: #dee2e6;
  color: #212529;
}

.cancel-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 編輯器佈局與極簡文字框 */
.code-editor-wrapper {
  flex: 1;
  display: flex;
  background: #23241f;
  overflow: hidden;
  position: relative;
}

.code-editor-textarea {
  width: 100%;
  height: 100%;
  background: #23241f;
  color: #f8f8f2;
  border: none;
  outline: none;
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  font-size: 0.82rem;
  line-height: 22px;
  padding: 20px 18px; /* 調整左側 padding 讓文字與行號完美靠攏對齊 */
  resize: none;
  box-sizing: border-box;
  tab-size: 4;
}

.code-editor-textarea:focus {
  background: #1e1f1c; /* 聚焦時稍微更暗，聚焦專注 */
}

/* 編輯器左側行號列 */
.editor-line-numbers {
  width: 55px;
  min-width: 55px;
  text-align: right;
  padding-right: 18px;
  color: #75715e;
  user-select: none;
  background: rgba(0, 0, 0, 0.15);
  border-right: 1px solid rgba(255, 255, 255, 0.05);
  overflow: hidden; /* 由 JS 同步滾動，隱藏滾動條 */
  padding-top: 20px;
  box-sizing: border-box;
}

.editor-line-number {
  height: 22px;
  line-height: 22px;
  width: 100%;
  box-sizing: border-box;
}

/* 編輯器高亮行號列標記 */
.highlight-editor-line {
  color: #f1c40f;
  background: rgba(255, 193, 7, 0.08);
  font-weight: 700;
}

/* 個資敏感資料高亮對照 HUD */
.editor-leak-hud {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(220, 53, 69, 0.04);
  border-bottom: 1px solid rgba(220, 53, 69, 0.15);
  padding: 10px 24px;
  font-size: 0.82rem;
  color: #f8f9fa;
  box-shadow: inset 0 -4px 10px rgba(0, 0, 0, 0.05);
}

.hud-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #adb5bd;
}

.hud-text strong {
  color: #ff4757;
}

/* 警示微動畫 */
.warning-flashing-icon {
  font-size: 1rem;
  animation: flash 1.5s ease-in-out infinite;
}

@keyframes flash {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

/* 敏感資料 Code Token */
.leak-highlight-token {
  font-family: 'Fira Code', Consolas, Monaco, monospace;
  background: rgba(220, 53, 69, 0.25);
  color: #ff4757;
  border: 1px solid rgba(220, 53, 69, 0.4);
  border-radius: 4px;
  padding: 2px 8px;
  font-weight: 700;
  margin-left: 6px;
  box-shadow: 0 0 6px rgba(220, 53, 69, 0.15);
}

/* 一鍵定位選取按鈕 */
.locate-btn {
  background: #dc3545;
  color: #fff;
  border: 1px solid #dc3545;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 4px;
}

.locate-btn:hover {
  background: #bd2130;
  border-color: #b21f2d;
  box-shadow: 0 2px 5px rgba(220, 53, 69, 0.3);
}
</style>
