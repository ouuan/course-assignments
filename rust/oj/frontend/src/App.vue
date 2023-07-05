<template>
  <nav-bar />
  <div class="container mx-auto px-4 sm:px-16 my-6">
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { toRef } from 'vue';
import { useRoute } from 'vue-router';

import NavBar from './components/NavBar.vue';

import useTitle from './composables/useTitle';

const route = useRoute();

useTitle(toRef(route, 'name'));
</script>
