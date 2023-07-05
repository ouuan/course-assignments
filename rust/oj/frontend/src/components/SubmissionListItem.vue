<template>
  <tr>
    <td>
      <router-link
        :to="`/submission/${job.id}`"
        class="link"
      >
        #{{ job.id }}
      </router-link>
    </td>
    <td>
      <router-link :to="`/submission/${job.id}`">
        <span
          :style="{
            color: `var(--theme-status-${statusText.toLowerCase().replace(/ /g, '-')})`
          }"
        >
          {{ statusText }}
        </span>
      </router-link>
    </td>
    <td>
      <router-link :to="`/submission/${job.id}`">
        <colored-score :score="job.score" />
      </router-link>
    </td>
    <td>
      <router-link
        :to="`${
          job.submission.contest_id ? `/contest/${job.submission.contest_id}` : ''
        }/problem/${job.submission.problem_id}`"
        class="link"
      >
        <api-result :fetch-results="[problem]">
          {{ problem.data.value.name }}
        </api-result>
      </router-link>
    </td>
    <td>
      <api-result :fetch-results="[user]">
        {{ user.data.value.name }}
      </api-result>
    </td>
    <td :title="`${totalTime}μs`">
      {{ Math.round(totalTime / 1000) }}ms
    </td>
    <td :title="`${totalMem}B`">
      {{ filesize(totalMem, {standard: 'iec', precision: 3}) }}
    </td>
    <td>
      {{ job.submission.language }}
    </td>
    <td :title="`${job.submission.source_code.length}B`">
      {{ filesize(job.submission.source_code.length, {standard: 'iec', precision: 3}) }}
    </td>
    <td :title="job.created_time">
      {{ format(new Date(job.created_time), 'yyyy-MM-dd HH:mm:ss') }}
    </td>
    <td
      class="hidden 2xl:table-cell"
      :title="job.updated_time"
    >
      {{ format(new Date(job.updated_time), 'yyyy-MM-dd HH:mm:ss') }}
    </td>
  </tr>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import filesize from 'filesize';
import { format } from 'date-fns';

import ApiResult from './ApiResult.vue';
import ColoredScore from './ColoredScore.vue';

import { Job } from '~/types';
import { useApi } from '~/composables/useApi';

const props = defineProps<{ job: Job }>();

const user = useApi(`/users/${props.job.submission.user_id}`);
const problem = useApi(`/problems/${props.job.submission.problem_id}`);

const statusText = computed(() => {
  if (props.job.state === 'Finished') {
    return props.job.result;
  }
  return props.job.state;
});

const totalTime = computed(() => props.job.cases.slice(1).reduce((sum, { time }) => sum + time, 0));
const totalMem = computed(
  () => props.job.cases.slice(1).reduce((sum, { memory }) => sum + memory, 0),
);
</script>
