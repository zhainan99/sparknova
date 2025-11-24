import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import App from "./App.vue";
import "./styles/global.css";

// 创建 Vue 应用实例
const app = createApp(App);

// 创建 Pinia 实例并注册持久化插件
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);

// 注册 Pinia
app.use(pinia);

// 挂载应用
app.mount("#app");
