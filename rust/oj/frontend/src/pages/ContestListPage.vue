<template>
  <api-result :fetch-results="[contests]">
    <div class="overflow-auto">
      <table
        v-if="contests.data.value.length"
        class="w-full"
      >
        <thead>
          <tr>
            <th>ID</th>
            <th>比赛</th>
            <th>开始时间</th>
            <th class="hidden lg:table-cell">
              结束时间
            </th>
            <th>时长</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="contest of contests.data.value.slice().reverse()"
            :key="contest.id"
          >
            <td>{{ contest.id }}</td>
            <td>
              <router-link
                :to="`/contest/${contest.id}`"
                class="link"
              >
                {{ contest.name }}
              </router-link>
            </td>
            <td>{{ format(new Date(contest.from), 'yyyy-MM-dd hh:mm:ss') }}</td>
            <td class="hidden lg:table-cell">
              {{ format(new Date(contest.to), 'yyyy-MM-dd hh:mm:ss') }}
            </td>
            <td>
              {{
                formatDuration(intervalToDuration({
                  start: new Date(contest.from),
                  end: new Date(contest.to),
                }), { locale: zhCN })
              }}
            </td>
          </tr>
        </tbody>
      </table>
      <div
        v-else
        class="text-center"
      >
        暂无比赛
      </div>
    </div>
  </api-result>
</template>

<script setup lang="ts">
import { format, intervalToDuration, formatDuration } from 'date-fns';
import { zhCN } from 'date-fns/locale';

import ApiResult from '~/components/ApiResult.vue';

import { useApi } from '~/composables/useApi';

const contests = useApi('/contests');
</script>

<style scoped lang="scss">
table {
  &:deep(td), &:deep(th) {
    @apply text-start px-4 py-2;
  }

  tbody:deep(tr) {
    @apply b-y-2;
  }
}
</style>
