<template>
  <api-result :fetch-results="[contest]">
    <h1 class="text-center text-xl font-bold">
      {{ contest.data.value.name }}
    </h1>
    <div class="flex justify-center">
      <span>{{ format(new Date(contest.data.value.from), 'yyyy-MM-dd hh:mm:ss') }}</span>
      ~
      <span>{{ format(new Date(contest.data.value.to), 'yyyy-MM-dd hh:mm:ss') }}</span>
    </div>
    <table class="w-full my-6">
      <thead>
        <tr>
          <th>#</th>
          <th>题目</th>
        </tr>
      </thead>
      <tbody>
        <contest-problem-item
          v-for="(problemId, index) of contest.data.value.problem_ids"
          :key="problemId"
          :contest-id="parseInt(id)"
          :problem-id="problemId"
          :problem-index="index"
        />
      </tbody>
    </table>
    <div class="my-6">
      <router-link
        :to="`/contest/${id}/submissions`"
        class="btn"
      >
        评测列表
      </router-link>
    </div>
    <rank-list
      :contest-id="id"
      :problem-ids="contest.data.value.problem_ids"
    />
  </api-result>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { format } from 'date-fns';

import ApiResult from '~/components/ApiResult.vue';
import ContestProblemItem from '~/components/ContestProblemItem.vue';
import RankList from '~/components/RankList.vue';

import { useApi } from '~/composables/useApi';
import useTitle from '~/composables/useTitle';

const props = defineProps<{
  id: string,
}>();

const contest = useApi(`/contests/${props.id}`);

useTitle(computed(() => (contest.data.value?.name ? `${contest.data.value.name} - 比赛` : `比赛 #${props.id}`)));
</script>

<style scoped lang="scss">
table {
  th, :deep(td) {
    @apply text-start p-2 first:w-10;
  }
}
</style>
