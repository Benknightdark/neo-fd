import { defineStore } from 'pinia';
import { ref } from 'vue';

// 定義通知類型：資訊、成功、警告、錯誤
export type NotificationType = 'info' | 'success' | 'warning' | 'error';

// 定義單一通知資料結構
export interface Notification {
  id: number;
  message: string;
  type: NotificationType;
}

// 全域通知訊息 Store，用於跨組件 Toast 提示
export const useNotificationStore = defineStore('notification', () => {
  // 活動中的通知列表
  const notifications = ref<Notification[]>([]);
  let nextId = 0;

  // 新增通知訊息，預設 5 秒後自動消失
  function add(message: string, type: NotificationType = 'info') {
    const id = nextId++;
    notifications.value.push({ id, message, type });

    // 5 秒後自動觸發移除
    setTimeout(() => {
      remove(id);
    }, 5000);
  }

  // 依 ID 移除指定通知
  function remove(id: number) {
    notifications.value = notifications.value.filter((n) => n.id !== id);
  }

  return {
    notifications,
    add,
    remove,
  };
});
