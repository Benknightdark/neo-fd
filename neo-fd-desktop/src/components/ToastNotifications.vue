<script setup lang="ts">
import { useNotificationStore } from '../stores/notification';

const store = useNotificationStore();
</script>

<template>
  <div class="toast-container" aria-live="polite">
    <TransitionGroup name="toast-list">
      <div
        v-for="toast in store.notifications"
        :key="toast.id"
        class="toast-item"
        :class="`toast-${toast.type}`"
      >
        <div class="toast-icon">
          <span v-if="toast.type === 'success'">✔</span>
          <span v-else-if="toast.type === 'error'">❌</span>
          <span v-else-if="toast.type === 'warning'">⚠️</span>
          <span v-else>ℹ</span>
        </div>
        <div class="toast-message">{{ toast.message }}</div>
        <button
          class="toast-close"
          @click="store.remove(toast.id)"
          aria-label="關閉通知"
        >
          &times;
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 350px;
  width: calc(100vw - 40px);
  pointer-events: none;
}

.toast-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-radius: 12px;
  pointer-events: auto;

  /* Glassmorphism styling */
  background: rgba(255, 255, 255, 0.75);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.35);

  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  position: relative;
  overflow: hidden;
}

/* Color accents based on HSL tailored colors */
.toast-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 4px;
}

.toast-success::before {
  background: linear-gradient(to bottom, hsl(142, 70%, 45%), hsl(142, 60%, 55%));
}

.toast-error::before {
  background: linear-gradient(to bottom, hsl(350, 80%, 50%), hsl(350, 70%, 60%));
}

.toast-warning::before {
  background: linear-gradient(to bottom, hsl(38, 90%, 50%), hsl(38, 80%, 60%));
}

.toast-info::before {
  background: linear-gradient(to bottom, hsl(200, 90%, 50%), hsl(200, 80%, 60%));
}

.toast-icon {
  font-size: 1.1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 24px;
}

.toast-success .toast-icon { color: hsl(142, 70%, 40%); }
.toast-error .toast-icon { color: hsl(350, 80%, 45%); }
.toast-warning .toast-icon { color: hsl(38, 90%, 45%); }
.toast-info .toast-icon { color: hsl(200, 90%, 40%); }

.toast-message {
  flex: 1;
  font-size: 0.9rem;
  font-weight: 500;
  color: #2c3e50;
  line-height: 1.4;
}

.toast-close {
  background: none;
  border: none;
  font-size: 1.25rem;
  color: #95a5a6;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  transition: all 0.2s;
}

.toast-close:hover {
  background: rgba(0, 0, 0, 0.05);
  color: #2c3e50;
}

/* Transition Group Animations */
.toast-list-enter-from {
  opacity: 0;
  transform: translateX(50px) scale(0.9);
}

.toast-list-leave-to {
  opacity: 0;
  transform: translateX(50px) scale(0.9);
}

.toast-list-leave-active {
  position: absolute;
  width: 100%;
}
</style>
