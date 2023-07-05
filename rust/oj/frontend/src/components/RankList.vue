<template>
  <div class="flex flex-wrap gap-3 my-3">
    <label>
      记分方式:
      <select v-model="scoringRule">
        <option
          v-for="rule of SCORING_RULES"
          :key="rule"
        >{{ rule }}</option>
      </select>
    </label>
    <label>
      打破平局:
      <select v-model="tieBreaker">
        <option :value="undefined" />
        <option
          v-for="breaker of TIE_BREAKERS"
          :key="breaker"
        >{{ breaker }}</option>
      </select>
    </label>
  </div>
  <api-result :fetch-results="[ranklist]">
    <div class="overflow-auto">
      <table class="w-full">
        <thead>
          <tr>
            <th>排名</th>
            <th>用户</th>
            <th>总分</th>
            <th
              v-for="(problemId, index) of problemIds"
              :key="problemId"
              class="link"
            >
              <router-link
                v-if="contestId"
                :to="`/contest/${contestId}/problem/${problemId}`"
              >
                {{ (index + 10).toString(36).toUpperCase() }}
              </router-link>
              <router-link
                v-else
                :to="`/problem/${problemId}`"
              >
                {{ problemId }}
              </router-link>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item of ranklist.data.value"
            :key="item.user.id"
            class="b-y-2"
          >
            <td>{{ item.rank }}</td>
            <td>{{ item.user.name }}</td>
            <td>
              <colored-score
                :score="item.scores.reduce((sum: number, score: number) => sum + score, 0)"
                :total="item.scores.length * 100"
              />
            </td>
            <td
              v-for="(score, index) of item.scores"
              :key="index"
            >
              <colored-score :score="score" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </api-result>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';

import ApiResult from './ApiResult.vue';
import ColoredScore from './ColoredScore.vue';

import { useApi } from '~/composables/useApi';

const props = defineProps<{
  contestId?: string,
  problemIds: number[],
}>();

const SCORING_RULES = ['latest', 'highest'] as const;
const TIE_BREAKERS = ['submission_time', 'submission_count', 'user_id'] as const;

const scoringRule = ref<(typeof SCORING_RULES)[number]>('latest');
const tieBreaker = ref<(typeof TIE_BREAKERS)[number]>();

const ranklist = useApi(computed(() => {
  const params = new URLSearchParams();
  params.set('scoring_rule', scoringRule.value);
  if (tieBreaker.value) {
    params.set('tie_breaker', tieBreaker.value);
  }
  return `/contests/${props.contestId || 0}/ranklist?${params}`;
}), { refetch: true });
</script>

<style scoped lang="scss">
table {
  td, th {
    @apply text-center p-2;
  }
}
</style>
