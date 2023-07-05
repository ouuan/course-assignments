import { defineConfig } from 'vite';
import unocss from 'unocss/vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

export default defineConfig({
  plugins: [
    unocss(),
    vue(),
  ],
  resolve: {
    alias: {
      '~': resolve(__dirname, 'src'),
    },
  },
  base: '',
  build: {
    target: 'es2016',
  },
});
