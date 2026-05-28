/// <reference types="vite/client" />

// 扩展 Window 对象
interface Window {
  macOS?: any;
  config?: any;
  tool?: any;
  loadingInterval?: ReturnType<typeof setInterval> | null;
}
