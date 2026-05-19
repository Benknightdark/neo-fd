import { defineStore } from 'pinia';
import { ref } from 'vue';

export type NotificationType = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
  id: number;
  message: string;
  type: NotificationType;
}

export const useNotificationStore = defineStore('notification', () => {
  const notifications = ref<Notification[]>([]);
  let nextId = 0;

  function add(message: string, type: NotificationType = 'info') {
    const id = nextId++;
    notifications.value.push({ id, message, type });

    // 5 秒後自動移除
    setTimeout(() => {
      remove(id);
    }, 5000);
  }

  function remove(id: number) {
    notifications.value = notifications.value.filter((n) => n.id !== id);
  }

  return {
    notifications,
    add,
    remove,
  };
});
