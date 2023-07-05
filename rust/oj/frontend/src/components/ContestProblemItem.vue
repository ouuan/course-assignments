<template>
  <tr class="b-y-2">
    <td>{{ (props.problemIndex + 10).toString(36).toUpperCase() }}</td>
    <td>
      <api-result :fetch-results="[problem]">
        <router-link
          :to="`/contest/${contestId}/problem/${problemId}`"
          class="link"
        >
          {{ problem.data.value.name }}
        </router-link>
      </api-result>
    </td>
  </tr>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import ApiResult from '~/components/ApiResult.vue';

import { useApi } from '~/composables/useApi';

const props = defineProps<{
  contestId: number,
  problemId: number,
  problemIndex: number,
}>();

const problem = useApi(computed(() => `/problems/${props.problemId}`));
</script>
