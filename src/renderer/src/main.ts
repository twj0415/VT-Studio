import { createApp } from 'vue';
import TDesign from 'tdesign-vue-next';
import { createPinia } from 'pinia';
import App from './App.vue';
import { i18n } from './i18n';
import { router } from './router';
import { useAppearanceStore } from './stores/appearance';
import { useLanguageStore } from './stores/language';
import './styles/index.scss';
import 'tdesign-vue-next/es/style/index.css';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);
app.use(i18n);
app.use(TDesign);
useLanguageStore(pinia).init();
useAppearanceStore(pinia).init();
app.mount('#app');
