<template>
  <div
    v-if="fetchResults.find((result) => result.isFetching.value)"
    class="flex items-center gap-1"
  >
    <span class="i-mdi-loading animate-spin" />
    <span>Loading...</span>
  </div>
  <div v-else-if="firstErrorResult">
    <div>API Error: {{ firstErrorResult.error.value }}</div>
    <div v-if="firstErrorResult.data.value">
      {{ firstErrorResult.data.value.message ?? firstErrorResult.data.value }}
    </div>
  </div>
  <div v-else-if="fetchResults.every((result) => result.data.value)">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { UseFetchReturn } from '@vueuse/core';

const props = defineProps<{
  fetchResults: UseFetchReturn<any>[],
}>();

const firstErrorResult = computed(() => props.fetchResults.find((result) => result.error.value));
</script>
