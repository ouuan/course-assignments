<template>
  <span :style="{ color: `hsl(${scoreHue}, 100%, 40%)`}">
    {{ Number(score.toFixed(3)) }}
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(defineProps<{
  score: number,
  total?: number,
}>(), {
  total: 100,
});

// https://github.com/vfleaking/uoj/blob/774870a4ec4f87437e2d89662538901dbf3160c0/web/public/js/uoj.js#L126-L134
const scoreHue = computed(() => {
  if (props.score >= props.total - 1e-10) {
    return 120;
  }
  if (props.score <= 0) {
    return 0;
  }
  return 30 + (props.score / props.total) * 60;
});
</script>
