import { createPinia } from 'pinia';
import { createApp } from 'vue';
import './style.css';
import App from './App.vue';

// 建立 Vue 應用程式實例
const app = createApp(App);

// 建立全域狀態管理器 Pinia 實例
const pinia = createPinia();

// 註冊 Pinia 狀態管理並掛載至 DOM
app.use(pinia);
app.mount('#app');
