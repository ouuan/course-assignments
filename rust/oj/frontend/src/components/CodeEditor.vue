<template>
  <code-mirror
    v-model:value="code"
    border
    original-style
    :options="cmOptions"
  />
</template>

<script setup lang="ts">
import CodeMirror from 'codemirror-editor-vue3';
import { EditorConfiguration } from 'codemirror';
import { useVModel } from '@vueuse/core';
import { computed } from 'vue';

import 'codemirror/mode/clike/clike.js';
import 'codemirror/mode/rust/rust.js';

const props = defineProps<{
  code: string,
  readOnly: boolean,
  language: string,
}>();

const emit = defineEmits<{
  (e: 'update:code', value: string): void,
}>();

const code = useVModel(props, 'code', emit);

const modeForLanguage = {
  Rust: 'rust',
  C: 'text/x-csrc',
  'C++': 'text/x-c++src',
};

const cmOptions = computed((): EditorConfiguration => ({
  mode: modeForLanguage[props.language as keyof typeof modeForLanguage] || 'null',
  indentUnit: 4,
  readOnly: props.readOnly,
  ...(props.readOnly ? {
    cursorBlinkRate: -1,
    inputStyle: 'textarea', // fix cursor not hidden in readonly editors on mobile
  } : {}),
}));
</script>
