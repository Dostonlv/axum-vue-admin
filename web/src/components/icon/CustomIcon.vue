<script setup>
/** Custom icon */
const props = defineProps({
  /** icon name (file name under assets/svg) */
  icon: {
    type: String,
    required: true,
  },
  size: {
    type: Number,
    default: 14,
  },
  color: {
    type: String,
    default: undefined,
  },
})
</script>

<template>
  <TheIcon type="custom" v-bind="props" />
</template>
