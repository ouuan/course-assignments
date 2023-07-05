<template>
  <api-result :fetch-results="[problem, languages, users]">
    <div>
      <div class="flex justify-center flex-wrap items-center gap-x-4 mb-6">
        <h1 class="text-xl font-bold">
          #{{ problem.data.value.id }}. {{ problem.data.value.name }}
        </h1>
        <div>(题目类型: {{ problem.data.value.problem_type }})</div>
      </div>
      <code-editor
        v-model:code="code"
        :read-only="false"
        :language="language"
      />
      <div class="flex flex-wrap items-center gap-4 my-3">
        <label>
          语言:
          <select v-model="language">
            <option
              v-for="lang of languages.data.value"
              :key="lang"
              :value="lang"
            >
              {{ lang }}
            </option>
          </select>
        </label>
        <label>
          用户:
          <select v-model="userId">
            <option
              v-for="user of users.data.value"
              :key="user.id"
              :value="user.id"
            >
              {{ user.name }}
            </option>
          </select>
        </label>
        <button
          class="btn"
          @click="submit"
        >
          提交
        </button>
      </div>
    </div>
  </api-result>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import ApiResult from '~/components/ApiResult.vue';
import CodeEditor from '~/components/CodeEditor.vue';

import { useApi, apiUrl } from '~/composables/useApi';
import useTitle from '~/composables/useTitle';

const props = defineProps<{
  id: string,
  contestId?: string,
}>();

const problem = useApi(`/problems/${props.id}`);
const languages = useApi('/languages');
const users = useApi('/users');

useTitle(computed(() => {
  const problemName = problem.data.value?.name ? `. ${problem.data.value?.name}` : '';
  const contest = props.contestId === undefined ? '' : ` - 比赛 #${props.contestId}`;
  return `题目 #${props.id}${problemName}${contest}`;
}));

const code = ref('');
const language = ref('');
const userId = ref(0);

watch(languages.data, (languageList) => {
  [language.value] = languageList;
});

const router = useRouter();

async function submit() {
  const data = {
    source_code: code.value,
    language: language.value,
    user_id: userId.value,
    contest_id: parseInt(props.contestId || '0', 10),
    problem_id: parseInt(props.id, 10),
  };
  try {
    const response = await fetch(apiUrl('/jobs').value, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(data),
    });
    if (response.status === 200) {
      const responseData = await response.json();
      router.push(`/submission/${responseData.id}`);
    } else {
      const responseText = await response.text();
      try {
        alert(`Failed to submit: ${JSON.parse(responseText).message}`);
      } catch (e) {
        alert(`Failed to submit: ${responseText}`);
      }
    }
  } catch (e) {
    alert(`Failed to submit: ${e}`);
  }
}
</script>
