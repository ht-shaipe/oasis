import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN.json'
import en from './en.json'

export type MessageSchema = typeof zhCN

const i18n = createI18n<[MessageSchema], 'zh-CN' | 'en'>({
  legacy: false, // Composition API 模式
  locale: localStorage.getItem('locale') || 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en': en,
  },
})

export default i18n
