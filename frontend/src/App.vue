<script setup lang="ts">
import { onMounted } from "vue";
import { useFaithStore } from "@/stores/faith";

const store = useFaithStore();

onMounted(async () => {
  await store.init();
});
</script>

<template>
  <router-view />
</template>

<style scoped>
</style>
