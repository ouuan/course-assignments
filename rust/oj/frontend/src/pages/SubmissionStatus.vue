<template>
  <api-result :fetch-results="[job]">
    <submission-list :jobs="[job.data.value]" />
    <code-editor
      :code="job.data.value.submission.source_code"
      :language="job.data.value.submission.language"
      read-only
      class="my-6"
    />
    <div class="overflow-auto">
      <table class="w-full">
        <thead>
          <tr>
            <th>测试点</th>
            <th>结果</th>
            <th>用时</th>
            <th>内存</th>
            <th>信息</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="testCase of job.data.value.cases"
            :key="testCase.id"
            class="b-y-2"
          >
            <td v-if="testCase.id">
              #{{ testCase.id }}
            </td>
            <td v-else>
              编译
            </td>
            <td>
              <span
                :style="{
                  color: `var(--theme-status-${testCase.result.toLowerCase().replace(/ /g, '-')})`
                }"
              >
                {{ testCase.result }}
              </span>
            </td>
            <td :title="`${testCase.time}μs`">
              {{ Math.round(testCase.time / 1000) }}ms
            </td>
            <td :title="`${testCase.memory}B`">
              {{ filesize(testCase.memory, { iec: true, precision: 3 }) }}
            </td>
            <td
              class="max-w-40 truncate"
              :title="testCase.info"
            >
              {{ testCase.info }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="my-6">
      <button
        v-if="job.data.value.state === 'Finished'"
        class="btn"
        @click="rejudge"
      >
        重测
      </button>
      <button
        v-if="job.data.value.state === 'Queueing'"
        class="btn"
        @click="cancel"
      >
        取消评测
      </button>
    </div>
  </api-result>
</template>

<script setup lang="ts">
import filesize from 'filesize';

import ApiResult from '~/components/ApiResult.vue';
import CodeEditor from '~/components/CodeEditor.vue';
import SubmissionList from '~/components/SubmissionList.vue';

import { apiUrl, useApi } from '~/composables/useApi';
import useTitle from '~/composables/useTitle';

const props = defineProps<{
  id: string,
}>();

useTitle(`提交 #${props.id}`);

const job = useApi(`/jobs/${props.id}`);

async function request(method: string) {
  const response = await fetch(apiUrl(`/jobs/${props.id}`).value, { method });
  if (response.status === 200) {
    window.location.reload();
  } else {
    const text = await response.text();
    try {
      alert(JSON.parse(text).message);
    } catch (e) {
      alert(text);
    }
  }
}

async function rejudge() {
  await request('PUT');
}

async function cancel() {
  await request('DELETE');
}
</script>

<style scoped lang="scss">
table {
  th, td {
    @apply text-center px-3 py-1;
  }
}
</style>
