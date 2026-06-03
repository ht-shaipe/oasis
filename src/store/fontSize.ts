import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type FontSize = 'small' | 'medium' | 'large'

export const useFontSizeStore = defineStore('fontSize', () => {
  const size = ref<FontSize>(
    (localStorage.getItem('fontSize') as FontSize) || 'medium'
  )

  const fontSizeMap = {
    small: 13,
    medium: 14,
    large: 16
  }

  const currentSize = ref(fontSizeMap[size.value])

  function setSize(s: FontSize) {
    size.value = s
    currentSize.value = fontSizeMap[s]
  }

  // 同步到 DOM 和 localStorage
  watch(size, (val) => {
    document.documentElement.style.setProperty('--font-size-base', `${fontSizeMap[val]}px`)
    localStorage.setItem('fontSize', val)
  }, { immediate: true })

  return { size, currentSize, setSize, fontSizeMap }
})