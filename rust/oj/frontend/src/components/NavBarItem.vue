<template>
  <li>
    <router-link
      class="flex-1 flex items-center gap-1 p-2 hover:bg-gray-3"
      :to="to"
    >
      <span :class="icon" />
      <span>{{ title }}</span>
    </router-link>
  </li>
</template>

<script setup lang="ts">
defineProps<{
  to: string,
  title: string,
  icon: string,
}>();
</script>
