import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from './app/App.vue'
import { router } from './app/router'
import { i18n } from './shared/i18n'
import './styles/reset.css'
import './styles/tailwind.css'
import './styles/global.scss'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)
app.mount('#app')
