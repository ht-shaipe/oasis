import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import i18n from '@/locales'

export type Locale = 'zh-CN' | 'en'

const trayLocaleMap: Record<Locale, { show: string; hide: string; about: string; quit: string }> = {
  'zh-CN': { show: '显示主窗口', hide: '隐藏主窗口', about: '关于 Oasis', quit: '退出' },
  'en': { show: 'Show Window', hide: 'Hide Window', about: 'About Oasis', quit: 'Quit' },
}

export const useLocaleStore = defineStore('locale', () => {
  const locale = ref<Locale>(
    (localStorage.getItem('locale') as Locale) || 'zh-CN'
  )

  function setLocale(l: Locale) {
    locale.value = l
  }

  function toggleLocale() {
    locale.value = locale.value === 'zh-CN' ? 'en' : 'zh-CN'
  }

  // 同步到 i18n 实例和 localStorage，并更新原生托盘菜单文字
  watch(locale, async (val) => {
    localStorage.setItem('locale', val)
    i18n.global.locale.value = val
    // 更新 Rust 端托盘菜单
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('update_tray_locale', { locale: trayLocaleMap[val] })
    } catch {
      // 非 Tauri 环境或命令未注册时忽略
    }
  }, { immediate: true })

  return { locale, setLocale, toggleLocale }
})
