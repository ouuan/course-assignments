<template>
  <div class="overflow-auto">
    <table
      v-if="jobs.length"
      class="w-full"
    >
      <thead>
        <tr>
          <th>ID</th>
          <th>状态</th>
          <th>分数</th>
          <th>题目</th>
          <th>用户</th>
          <th>用时</th>
          <th>内存</th>
          <th>语言</th>
          <th>码长</th>
          <th>提交时间</th>
          <th class="hidden 2xl:table-cell">
            更新时间
          </th>
        </tr>
      </thead>
      <tbody>
        <submission-list-item
          v-for="job of jobs"
          :key="job.id"
          :job="job"
        />
      </tbody>
    </table>
    <div
      v-else
      class="text-center"
    >
      没有找到符合要求的提交
    </div>
  </div>
</template>

<script setup lang="ts">
import SubmissionListItem from './SubmissionListItem.vue';

import { Job } from '~/types';

defineProps<{ jobs: Job[] }>();
</script>

<style scoped lang="scss">
table {
  &:deep(td), &:deep(th) {
    @apply text-center px-4 py-2;
  }

  tbody:deep(tr) {
    @apply b-y-2;
  }
}
</style>
