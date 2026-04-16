import { defineStore } from 'pinia'
import { getAction, getInitialState, getLegalActions, getNextState, getTurnedState, won, lost } from 'quantum-animal-shogi-webasm'
import type { Action, State } from 'quantum-animal-shogi-webasm'
import { computed, nextTick, ref } from 'vue'
import { filterMap, map, pipe } from 'rambda'
import chickUrl from '@/assets/chick.bmp'
import chickenUrl from '@/assets/chicken.bmp'
import elephantUrl from '@/assets/elephant.bmp'
import giraffeUrl from '@/assets/giraffe.bmp'
import lionUrl from '@/assets/lion.bmp'

function* getBits(x: number): Iterable<number> {
  while (x) {
    yield 31 - Math.clz32(x & -x)

    x &= x - 1
  }
}

export const useQuantumAnimalShogiStore = defineStore('state', () => {
  const state        = ref(getInitialState())
  const isMyTurn     = ref(true)
  const reward       = ref(0)
  const action0      = ref<number | null>(null)
  const action1      = ref<number | null>(null)
  const animalImages = ref<HTMLImageElement[] | null>(null)

  const loadImage = (src: string): Promise<HTMLImageElement> => {
    return new Promise(resolve => {
      const image = new Image()

      image.onload = () => resolve(image)
      image.src = src
    })
  }

  const initialize = async () => {
    animalImages.value = await Promise.all(
      pipe(
        [chickUrl, giraffeUrl, elephantUrl, lionUrl, chickenUrl],
        map(async (url) => await loadImage(url))
      )
    )
  }

  const setPieceState = (state: State, pieceIndex: number, pieceState: number[]) => {
    for (const bit of getBits(state.pieces[pieceIndex]!)) {
      pieceState[bit] = 1
    }

    pieceState[5] = pieceIndex < 4 ? 1 : 0
    pieceState[5 + 1] = state.ownership & (1 << pieceIndex) ? 1 : 0
  }

  const getBoard = (state: State) => {
    const result = Array.from({ length: 4 * 3 }, () => Array.from({ length: 5 + 2 }, () => 0))

    for (const [pieceIndex, bitBoard] of pipe(
      [...state.bitBoards],
      filterMap((bitBoard, pieceIndex) => bitBoard !== 0 ? [pieceIndex, bitBoard] as [number, number] : null)
    )) {
      setPieceState(state, pieceIndex, result[(31 - Math.clz32(bitBoard & -bitBoard))]!)
    }

    return result
  }

  const getHands = (state: State, ownership: boolean) => {
    const result = Array.from({ length: 8 }, () => Array.from({ length: 5 + 1 + 1 }, () => 0))

    for (const [pieceIndex, i] of pipe(
      [...state.bitBoards],
      filterMap((bitBoard, pieceIndex) => !bitBoard && ((state.ownership & (1 << pieceIndex)) !== 0) === ownership ? {value: pieceIndex} : null),
      map((pieceIndex, i): [number, number] => [pieceIndex.value, i])
    )) {
      setPieceState(state, pieceIndex, result[i]!)
    }

    return result
  }

  const board        = computed(() => getBoard(isMyTurn.value ? state.value : getTurnedState(state.value)))
  const allyHands    = computed(() => getHands(isMyTurn.value ? state.value : getTurnedState(state.value), true ))
  const enemyHands   = computed(() => getHands(isMyTurn.value ? state.value : getTurnedState(state.value), false))
  const legalActions = computed(() => getLegalActions(state.value))

  const reset = () => {
    state.value = getInitialState()
  }

  const step = async (action: Action) => {
    state.value = getNextState(state.value, action)
    isMyTurn.value = !isMyTurn.value

    if (won(state.value)) {
      reward.value = isMyTurn.value ? -1 : 1
    }

    if (lost(state.value)) {
      reward.value = isMyTurn.value ? 1 : -1
    }

    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 100))
  }

  const executeAction = async () => {
    const action = [action0.value!, action1.value!] as [number, number]

    action0.value = null
    action1.value = null

    await step(action)

    if (reward.value != 0) {
      return
    }

    await step(getAction(state.value))

    if (reward.value != 0) {
      return
    }
  }

  return { initialize, isMyTurn, reward, action0, action1, animalImages, board, allyHands, enemyHands, legalActions, reset, executeAction }
})
