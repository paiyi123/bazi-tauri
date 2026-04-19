import { createApp } from "vue";
import ElementPlus from "element-plus";
import zhTw from "element-plus/es/locale/lang/zh-tw";
import "element-plus/dist/index.css";
import App from "./App.vue";
import "./styles.css";

createApp(App).use(ElementPlus, { locale: zhTw }).mount("#app");
