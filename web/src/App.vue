<script setup lang="ts">
import GameBoard from '@/components/GameBoard.vue'
import { useQuantumAnimalShogiStore } from '@/stores/QuantumAnimalShogiStore';
import { onMounted, ref } from 'vue';

const store = useQuantumAnimalShogiStore()
const initialized = ref(false)

onMounted(async () => {
  await store.initialize()
  initialized.value = true
})
</script>

<template>
  <main>
    <GameBoard v-if="initialized" />
  </main>
</template>

<style>
body {
  font-family: Arial;
  font-size: 14px;
}
</style>
