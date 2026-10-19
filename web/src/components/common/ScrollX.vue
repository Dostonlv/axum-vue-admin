<template>
  <div ref="wrapper" class="wrapper" @mousewheel.prevent="handleMouseWheel">
    <template v-if="showArrow && isOverflow">
      <div class="left" @click="handleMouseWheel({ wheelDelta: 120 })">
        <icon-ic:baseline-keyboard-arrow-left />
      </div>
      <div class="right" @click="handleMouseWheel({ wheelDelta: -120 })">
        <icon-ic:baseline-keyboard-arrow-right />
      </div>
    </template>

    <div
      ref="content"
      v-resize="refreshIsOverflow"
      class="content"
      :class="{ overflow: isOverflow && showArrow }"
      :style="{
        transform: `translateX(${translateX}px)`,
      }"
    >
      <slot />
    </div>
  </div>
</template>

<script setup>
import { debounce, useResize } from '@/utils'

defineProps({
  showArrow: {
    type: Boolean,
    default: true,
  },
})

const translateX = ref(0)
const content = ref(null)
const wrapper = ref(null)
const isOverflow = ref(false)

const refreshIsOverflow = debounce(() => {
  const wrapperWidth = wrapper.value?.offsetWidth
  const contentWidth = content.value?.offsetWidth
  isOverflow.value = contentWidth > wrapperWidth
  resetTranslateX(wrapperWidth, contentWidth)
}, 200)

function handleMouseWheel(e) {
  const { wheelDelta } = e
  const wrapperWidth = wrapper.value?.offsetWidth
  const contentWidth = content.value?.offsetWidth
  /**
   * @wheelDelta horizontal scroll delta; >0 moves right, <0 moves left
   * @translateX translateX of the content
   * @wrapperWidth width of the container
   * @contentWidth width of the content
   */
  if (wheelDelta < 0) {
    if (wrapperWidth > contentWidth && translateX.value < -10) return
    if (wrapperWidth <= contentWidth && contentWidth + translateX.value - wrapperWidth < -10) return
  }
  if (wheelDelta > 0 && translateX.value > 10) {
    return
  }

  translateX.value += wheelDelta
  resetTranslateX(wrapperWidth, contentWidth)
}

const resetTranslateX = debounce(function (wrapperWidth, contentWidth) {
  if (!isOverflow.value) {
    translateX.value = 0
  } else if (-translateX.value > contentWidth - wrapperWidth) {
    translateX.value = wrapperWidth - contentWidth
  } else if (translateX.value > 0) {
    translateX.value = 0
  }
}, 200)

const observer = ref(null)
onMounted(() => {
  refreshIsOverflow()

  observer.value = useResize(document.body, refreshIsOverflow)
})
onBeforeUnmount(() => {
  observer.value?.disconnect()
})

function handleScroll(x, width) {
  const wrapperWidth = wrapper.value?.offsetWidth
  const contentWidth = content.value?.offsetWidth
  if (contentWidth <= wrapperWidth) return

  // When x is below the visible minimum
  if (x < -translateX.value + 150) {
    translateX.value = -(x - 150)
    resetTranslateX(wrapperWidth, contentWidth)
  }

  // When x is above the visible maximum
  if (x + width > -translateX.value + wrapperWidth) {
    translateX.value = wrapperWidth - (x + width)
    resetTranslateX(wrapperWidth, contentWidth)
  }
}

defineExpose({
  handleScroll,
})
</script>

<style lang="scss" scoped>
.wrapper {
  display: flex;
  background-color: #fff;

  z-index: 9;
  overflow: hidden;
  position: relative;
  .content {
    padding: 0 10px;
    display: flex;
    align-items: center;
    flex-wrap: nowrap;
    transition: transform 0.5s;
    &.overflow {
      padding-left: 30px;
      padding-right: 30px;
    }
  }
  .left,
  .right {
    background-color: #fff;
    position: absolute;
    top: 0;
    bottom: 0;
    margin: auto;

    width: 20px;
    height: 35px;
    display: flex;
    align-items: center;
    justify-content: center;

    font-size: 18px;
    border: 1px solid #e0e0e6;
    border-radius: 2px;

    z-index: 2;
    cursor: pointer;
  }
  .left {
    left: 0;
  }
  .right {
    right: 0;
  }
}
</style>
