import { createApp } from 'vue';

import '@unocss/reset/tailwind.css';
import 'uno.css';
import '~/styles/index.scss';

import router from './router';
import App from './App.vue';

createApp(App).use(router).mount('#app');
