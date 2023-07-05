<template>
  <api-result
    :fetch-results="[problemsResponse]"
  >
    <div>
      <table
        v-if="problems"
        class="w-full"
      >
        <thead>
          <tr>
            <th>#</th>
            <th>题目</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="problem of problems"
            :key="problem.id"
            class="b-y-2"
          >
            <td>{{ problem.id }}</td>
            <td>
              <router-link
                :to="`/problem/${problem.id}`"
                class="link"
              >
                {{ problem.name }}
              </router-link>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else>
        Api Error: Invalid response format
      </div>
    </div>
  </api-result>
</template>

<script setup lang="ts">
import { computed } from 'vue';

import ApiResult from '~/components/ApiResult.vue';

import { useApi } from '~/composables/useApi';

const problemsResponse = useApi('/problems');

interface Problem {
  id: number,
  name: string,
}

const problems = computed(() => {
  if (!problemsResponse.data.value) {
    return null;
  }
  const json = problemsResponse.data.value;
  if (!Array.isArray(json)) {
    return null;
  }
  const result: Problem[] = [];
  for (const item of json) {
    if (typeof item !== 'object' || item == null) {
      return null;
    }
    const { id, name } = item;
    if (typeof id !== 'number' || typeof name !== 'string') {
      return null;
    }
    result.push({ id, name });
  }
  return result;
});
</script>

<style scoped lang="scss">
table {
  td, th {
    @apply text-start px-4 py-1 first:w-10;
  }
}
</style>
