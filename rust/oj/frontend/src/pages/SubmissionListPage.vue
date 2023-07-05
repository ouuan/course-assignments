<template>
  <div v-if="contestId">
    <router-link
      :to="`/contest/${contestId}`"
      class="btn"
    >
      回到比赛 #{{ contestId }}
    </router-link>
  </div>
  <div class="flex flex-wrap my-6 gap-2">
    <label>
      用户:
      <input
        v-model.lazy="userName"
        type="text"
        class="b-2 rd-1 max-w-20"
      >
    </label>
    <label>
      题目:
      <input
        v-model.lazy="problemId"
        type="number"
        class="b-2 rd-1 max-w-20"
      >
    </label>
    <label>
      语言:
      <input
        v-model.trim.lazy="language"
        type="text"
        class="b-2 rd-1 max-w-20"
      >
    </label>
    <label>
      状态:
      <select v-model="state">
        <option :value="undefined" />
        <option
          v-for="s of STATES"
          :key="s"
        >{{ s }}</option>
      </select>
    </label>
    <label>
      结果:
      <select v-model="result">
        <option :value="undefined" />
        <option
          v-for="r of RESULTS"
          :key="r"
        >{{ r }}</option>
      </select>
    </label>
    <div>
      <label>
        从:
        <input
          v-model="from"
          type="datetime-local"
          class="b-2 rd-1"
        >
      </label>
      <label>
        到:
        <input
          v-model="to"
          type="datetime-local"
          class="b-2 rd-1"
        >
      </label>
    </div>
  </div>
  <api-result :fetch-results="[jobs]">
    <submission-list :jobs="jobs.data.value.slice().reverse()" />
  </api-result>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import ApiResult from '~/components/ApiResult.vue';
import SubmissionList from '~/components/SubmissionList.vue';

import { useApi } from '~/composables/useApi';

const props = defineProps<{
  contestId?: string,
}>();

const router = useRouter();

const jobs = useApi(
  computed(() => {
    const urlQuery = router.currentRoute.value.fullPath.split('?')[1];
    const contestQuery = props.contestId === undefined ? '' : `contest_id=${props.contestId}`;
    if (urlQuery) {
      return `/jobs?${urlQuery}&${contestQuery}`;
    }
    return `/jobs?${contestQuery}`;
  }),
  { refetch: true },
);

const STATES = ['Queueing', 'Running', 'Finished', 'Canceled'];
const RESULTS = [
  'Accepted',
  'Wrong Answer',
  'Time Limit Exceeded',
  'Runtime Error',
  'Memory Limit Exceeded',
  'Compilation Error',
  'Waiting',
  'Running',
  'SPJ Error',
  'System Error',
];

const { query: urlQuery } = router.currentRoute.value;
const userName = ref<string | undefined>(urlQuery.user_name?.toString());
const problemId = ref<string | number | undefined>(urlQuery.problem_id?.toString());
const language = ref<string | undefined>(urlQuery.language?.toString());
const from = ref<string | undefined>(urlQuery.from?.toString());
const to = ref<string | undefined>(urlQuery.to?.toString());
const state = ref<string | undefined>(urlQuery.state?.toString());
const result = ref<string | undefined>(urlQuery.result?.toString());

function query() {
  const parsedProblemId = parseInt(`${problemId.value}`, 10);
  router.push({
    path: router.currentRoute.value.path,
    query: {
      user_name: userName.value || undefined,
      problem_id: Number.isNaN(parsedProblemId) ? undefined : parsedProblemId,
      language: language.value || undefined,
      from: from.value ? new Date(from.value).toISOString() : undefined,
      to: to.value ? new Date(to.value).toISOString() : undefined,
      state: state.value || undefined,
      result: result.value || undefined,
    },
  });
}

watch([userName, problemId, language, from, to, state, result], query);
</script>
