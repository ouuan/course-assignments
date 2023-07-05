<template>
  <h1 class="text-center text-xl font-bold mb-6">
    总排行
  </h1>
  <api-result :fetch-results="[problems]">
    <rank-list :problem-ids="problemIds" />
  </api-result>
</template>

<script setup lang="ts">
import { computed } from 'vue';

import ApiResult from '~/components/ApiResult.vue';
import RankList from '~/components/RankList.vue';

import { useApi } from '~/composables/useApi';

const problems = useApi('/problems');
const problemIds = computed(() => {
  if (!Array.isArray(problems.data.value)) {
    return [];
  }
  return problems.data.value.map(({ id }) => id);
});
</script>
