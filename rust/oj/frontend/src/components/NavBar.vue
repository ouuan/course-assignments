<template>
  <header class="bg-gray-2 shadow">
    <div class="flex flex-wrap container mx-auto px-4 sm:px-16">
      <router-link
        class="flex items-center p-2 hover:bg-gray-3"
        to="/"
      >
        <span>Rust Course Online Judge</span>
      </router-link>
      <nav>
        <ul class="flex">
          <nav-bar-item
            to="/"
            title="首页"
            icon="xs:i-mdi-home"
          />
          <nav-bar-item
            to="/problems"
            title="题库"
            icon="xs:i-mdi-list-box"
          />
          <nav-bar-item
            to="/submissions"
            title="评测"
            icon="xs:i-mdi-timer-sand-full"
          />
          <nav-bar-item
            to="/contests"
            title="比赛"
            icon="xs:i-mdi-trophy"
          />
          <nav-bar-item
            to="/ranklist"
            title="排名"
            icon="xs:i-mdi-account-multiple"
          />
        </ul>
      </nav>
    </div>
  </header>
</template>

<script setup lang="ts">
import NavBarItem from './NavBarItem.vue';
</script>
