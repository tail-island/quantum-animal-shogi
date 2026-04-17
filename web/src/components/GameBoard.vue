<script setup lang="ts">
import { useQuantumAnimalShogiStore } from '@/stores/QuantumAnimalShogiStore'
import PieceCell from './PieceCell.vue'

const store = useQuantumAnimalShogiStore()
</script>

<template>
  <div class="grid">
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[1]!" :index="null" /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[0]!" :index="null" /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[11]!"     :index="11"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[10]!"     :index="10"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[9]!"      :index=" 9"   /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[6]!"  :index="18"   /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[7]!"  :index="19"   /></div>

    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[3]!" :index="null" /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[2]!" :index="null" /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[8]!"      :index=" 8"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[7]!"      :index=" 7"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[6]!"      :index=" 6"   /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[4]!"  :index="16"   /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[5]!"  :index="17"   /></div>

    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[5]!" :index="null" /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[4]!" :index="null" /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[5]!"      :index=" 5"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[4]!"      :index=" 4"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[3]!"      :index=" 3"   /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[2]!"  :index="14"   /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[3]!"  :index="15"   /></div>

    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[7]!" :index="null" /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.enemyHands[6]!" :index="null" /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[2]!"      :index=" 2"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[1]!"      :index=" 1"   /></div>
    <div class="cell piece-cell board-cell"><PieceCell :piece-state="store.board[0]!"      :index=" 0"   /></div>
    <div class="cell blank-cell"></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[0]!"  :index="12"   /></div>
    <div class="cell piece-cell hand-cell" ><PieceCell :piece-state="store.allyHands[1]!"  :index="13"   /></div>
  </div>
  <p class="reward" v-if="store.reward">reward = {{ store.reward }}</p>
</template>

<style scoped>
.grid {
  display: grid;
  grid-template-columns: repeat(9, 1fr);
  width: 100%;
}

.cell {
  aspect-ratio: 1 / 1;
}

.piece-cell {
  background-color: #c0c0c0;
}

.blank-cell {
  background-color: #808080;
}

.board-cell {
  border: 1px solid #000000;
}

.hand-cell {
  border: 1px solid #c0c0c0;
}

.reward {
  text-align: center;;
}
</style>
