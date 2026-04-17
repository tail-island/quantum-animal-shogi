<script setup lang="ts">
import { all, any, pipe } from 'rambda'
import { computed, onMounted, ref, watch } from 'vue'
import { useQuantumAnimalShogiStore } from '@/stores/QuantumAnimalShogiStore'

const props = defineProps<{
  pieceState: number[],
  index: number | null
}>()

const store = useQuantumAnimalShogiStore()
const canvas = ref<HTMLCanvasElement  | null>(null)

const isAction1Candidate = computed(() => {
  if (!store.action0 === null) {
    return false
  }

  return pipe(
    store.legalActions,
    any(action => action[0] === store.action0 && action[1] === props.index)
  )
})

const isBlank = () => {
  return pipe(props.pieceState, all(bit => !bit))
}

const draw = () => {
  const context = canvas.value!.getContext('2d')!

  context.clearRect(0, 0, canvas.value!.width, canvas.value!.height)

  if (isBlank()) {
    return
  }

  const size = canvas.value!.width - 1

  context.save()

  if (!props.pieceState[5 + 1]) {
    context.translate( 1 * 1 / 2 * size + 0.5,  1 * 1 / 2 * size + 0.5)
    context.rotate(Math.PI)
    context.translate(-1 * 1 / 2 * size - 0.5, -1 * 1 / 2 * size - 0.5)
  }

  context.beginPath()
  context.moveTo(1 / 2 * size + 0.5, 0 + 0.5)
  context.lineTo(0 + 0.5, 1 / 3 * size + 0.5)
  context.lineTo(0 + 0.5, size + 0.5)
  context.lineTo(size + 0.5, size + 0.5)
  context.lineTo(size + 0.5, 1 / 3 * size + 0.5)
  context.closePath()

  context.fillStyle = '#ffffff'
  context.fill()

  context.strokeStyle = '#404040'
  context.stroke()

  context.beginPath()
  context.arc(1 / 2 * size + 0.5, 1 / 2 * size + 0.5, 1 / 6 * size, 0, 2 * Math.PI)

  context.fillStyle = props.pieceState[5] ? '#ff8080' : '#80ff80'
  context.fill()

  for (const [index, column, row] of [
    [0, 0, 2],
    [1, 1, 2],
    [2, 2, 2],
    [3, 0, 1],
    [4, 2, 1]
  ] as [number, number, number][]) {
    if (!props.pieceState[index]) {
      continue
    }

    context.drawImage(store.animalImages![index]!, column / 3 * size + 0.5, row / 3 * size + 0.5, 1 / 3 * size + 0.5, 1 / 3 * size + 0.5)
  }

  context.restore()
}

const handleClick = () => {
  if (!store.isMyTurn || store.reward !== 0) {
    return
  }

  if (store.action0 === props.index) {
    store.action0 = null
    return
  }

  if (isAction1Candidate.value) {
    store.action1 = props.index!
    store.executeAction()
    return
  }

  if (pipe(store.legalActions, all(action => action[0] !== props.index))) {
    return
  }

  store.action0 = props.index!
}

onMounted(async () => {
  draw()
})

watch(
  () => props.pieceState,
  () => draw()
)
</script>

<template>
  <div class="cell" @click="handleClick">
    <div class="piece">
      <canvas ref="canvas" class="canvas" width="192" height="192"></canvas>
    </div>
    <div class="action-1-candidate" v-if="isAction1Candidate"></div>
  </div>
</template>

<style scoped>
.cell {
  box-sizing: border-box;
  position: relative;
  height: 100%;
  width: 100%;
}

.piece {
  position: absolute;
  inset: 0;
}

.canvas {
  height: 100%;
  width: 100%;
}

.action-1-candidate {
  position: absolute;
  background-color: rgba(128, 128, 255, 0.5);
  inset: 0;
}
</style>
