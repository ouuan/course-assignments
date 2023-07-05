import {
  defineConfig,
  presetIcons,
  presetUno,
} from 'unocss';
import transformerDirective from '@unocss/transformer-directives';

export default defineConfig({
  presets: [
    presetIcons(),
    presetUno(),
  ],
  theme: {
    breakpoints: {
      xs: '24em',
      sm: '40em',
      md: '48em',
      lg: '64em',
      xl: '80em',
      '2xl': '96em',
    },
  },
  shortcuts: {
    link: 'text-blue-7 hover:text-blue-5 active:text-blue-9',
    btn: 'bg-blue-3 hover:bg-blue-2 active:bg-blue-4 rd-1 px-2 py-1',
  },
  transformers: [
    transformerDirective(),
  ],
});
