/// <reference types="vite/client" />

import 'vue-i18n';

// 扩展 Window 对象
interface Window {
  macOS?: any;
  config?: any;
  tool?: any;
  loadingInterval?: ReturnType<typeof setInterval> | null;
}

// vue-i18n 类型声明
declare module 'vue-i18n' {
  export interface DefineLocaleMessage {}
}

export {};
