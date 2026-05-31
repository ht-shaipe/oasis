import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>(
    (localStorage.getItem('theme') as ThemeMode) || 'light'
  )

  const isDark = ref(mode.value === 'dark')

  function toggle() {
    mode.value = mode.value === 'light' ? 'dark' : 'light'
    isDark.value = mode.value === 'dark'
  }

  function setTheme(m: ThemeMode) {
    mode.value = m
    isDark.value = m === 'dark'
  }

  // 同步到 DOM 和 localStorage
  // 同时设置 data-theme 属性（自定义 CSS 变量）和 class（Element Plus dark mode）
  watch(mode, (val) => {
    document.documentElement.setAttribute('data-theme', val)
    document.documentElement.classList.toggle('dark', val === 'dark')
    localStorage.setItem('theme', val)
  }, { immediate: true })

  return { mode, isDark, toggle, setTheme }
})
