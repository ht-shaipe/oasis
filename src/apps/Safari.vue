<template>
    <MacWindow
        ref="macWindowRef"
        title="Safari"
        :isMinimized="isMinimized"
        @close="closeApp"
        :width="800"
        :height="600"
        @minimize="toggleMinimize"
        :startMaximized="false">
        <div class="p-0 flex flex-col h-full">
            <div
                class="flex items-center bg-[var(--color-bg)] border-b border-[var(--color-window-titlebar-border)] px-2.5 py-2">
                <div class="flex gap-2.5 mr-4">
                    <el-icon>
                        <ArrowLeft />
                    </el-icon>
                    <el-icon>
                        <ArrowRight />
                    </el-icon>
                </div>
                <div
                    class="flex-1 flex items-center bg-[var(--color-input-bg)] rounded-1.5 px-2.5 h-7 mx-2.5 text-[var(--app-font-13)] text-[var(--color-text-primary)]">
                    <el-icon class="mr-1.5 text-[var(--app-font-14)] text-[#00890a] shrink-0">
                        <Lock />
                    </el-icon>
                    <input
                        v-model="addressInput"
                        class="flex-1 min-w-0 border-none outline-none bg-transparent text-[var(--color-text-primary)] text-[var(--app-font-13)] h-full placeholder:text-[var(--color-text-tertiary)]"
                        @keyup.enter="navigateToUrl"
                        @focus="($event.target as HTMLInputElement).select()"
                        placeholder="输入网址或搜索" />
                </div>
                <div class="ml-2.5">
                    <el-button @click="toggleViewMode" size="small" type="primary" v-if="externalUrl">
                        {{ useIframe ? '原生WebView' : '内嵌Iframe' }}
                    </el-button>
                    <el-icon @click="toggleViewMode" class="cursor-pointer" v-else>
                        <Share />
                    </el-icon>
                </div>
            </div>
            <div class="flex-1 h-[calc(100%-3.25rem)]">
                <iframe v-if="useIframe && externalUrl" :src="externalUrl" class="w-full h-full border-0"></iframe>
                <iframe v-else-if="!externalUrl" :srcdoc="previewHtml" class="w-full h-full border-0"></iframe>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';
import { ArrowLeft, ArrowRight, Lock, Share } from '@element-plus/icons-vue';
import MacWindow from '@/components/common/MacWindow.vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

// 定义属性
const props = defineProps({
    isMinimized: {
        type: Boolean,
        default: false,
    },
    code: {
        type: String,
        default: '',
    },
});

// 事件发射
const emit = defineEmits(['close', 'minimize']);

// 状态变量
const externalUrl = ref('');
const currentUrl = ref('https://www.htui.tech');
const addressInput = ref('');
const useIframe = ref(true);
const isNativeMode = ref(false); // 是否正在使用 Rust 端嵌入式 WebView

// 导航到用户输入的 URL
const navigateToUrl = () => {
    let raw = addressInput.value.trim();
    if (!raw) return;
    if (!/^https?:\/\//i.test(raw)) {
        raw = 'https://' + raw;
    }
    // 如果当前在原生 WebView 模式，直接用新的 WebView 替换
    if (isNativeMode.value) {
        openExternalUrlInWebview(raw);
        return;
    }
    externalUrl.value = raw;
    currentUrl.value = new URL(raw).hostname;
    useIframe.value = true;
};

// 同步地址栏到 currentUrl 或 externalUrl
const syncAddressInput = () => {
    if (externalUrl.value) {
        addressInput.value = externalUrl.value;
    } else {
        addressInput.value = currentUrl.value;
    }
};

// ── 嵌入式 WebView（Rust 端，参照 crawler wry build_as_child）───

/// 获取 .safari-content 相对于主窗口的屏幕绝对坐标
const getWebviewBounds = async (): Promise<{ x: number; y: number; width: number; height: number } | null> => {
    const el = document.querySelector('.safari-content') as HTMLElement;
    if (!el) return null;
    const rect = el.getBoundingClientRect();
    const window = getCurrentWindow();
    const pos = await window.outerPosition();
    const scale = await window.scaleFactor();
    // outerPosition 是物理像素，getBoundingClientRect 是 CSS 像素，需要统一
    return {
        x: pos.x / scale + rect.x,
        y: pos.y / scale + rect.y,
        width: rect.width,
        height: rect.height,
    };
};

const syncWebviewBounds = async () => {
    if (!isNativeMode.value) return;
    const bounds = await getWebviewBounds();
    if (!bounds) return;
    invoke('update_embedded_webview_bounds', {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
    }).catch(() => {});
};

const openExternalUrlInWebview = async (url: string) => {
    try {
        const bounds = await getWebviewBounds();
        if (!bounds) throw new Error('.safari-content not found');

        await invoke('create_embedded_webview', {
            url,
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        });

        isNativeMode.value = true;
        useIframe.value = false;
        currentUrl.value = new URL(url).hostname;
        syncAddressInput();
    } catch (error) {
        console.error('Failed to create embedded webview:', error);
        externalUrl.value = url;
        currentUrl.value = new URL(url).hostname;
        useIframe.value = true;
        isNativeMode.value = false;
        syncAddressInput();
    }
};

const closeNativeWebview = () => {
    invoke('close_embedded_webview').catch(() => {});
    isNativeMode.value = false;
};

let resizeObserver: ResizeObserver | null = null;
let unlistenMove: (() => void) | null = null;

onMounted(async () => {
    const url = localStorage.getItem('safariUrl');
    if (url) {
        openExternalUrlInWebview(url);
        localStorage.removeItem('safariUrl');
    }
    syncAddressInput();

    // DOM 尺寸变化 → 同步 bounds
    const contentEl = document.querySelector('.safari-content');
    if (contentEl) {
        resizeObserver = new ResizeObserver(() => syncWebviewBounds());
        resizeObserver.observe(contentEl);
    }

    // 窗口拖动 → 同步 bounds（add_child 使用屏幕坐标，需手动跟随）
    unlistenMove = await getCurrentWindow().onMoved(() => {
        syncWebviewBounds();
    });

    navigateToUrl();
});

onUnmounted(() => {
    closeNativeWebview();
    resizeObserver?.disconnect();
    unlistenMove?.();
});

// 切换视图模式
const toggleViewMode = () => {
    if (externalUrl.value) {
        if (useIframe.value) {
            openExternalUrlInWebview(externalUrl.value);
        } else {
            closeNativeWebview();
            useIframe.value = true;
        }
    } else {
        console.log('没有外部URL可打开');
    }
};

// 关闭应用
const closeApp = () => {
    closeNativeWebview();
    emit('close');
};

// 切换最小化状态
const toggleMinimize = () => {
    emit('minimize');
};

// MacWindow 组件引用
const macWindowRef = ref<InstanceType<typeof MacWindow> | null>(null);

// 暴露 bringToFront 方法
defineExpose({
    bringToFront: () => macWindowRef.value?.bringToFront(),
});

// 计算属性：预览HTML
const previewHtml = computed(() => {
    // 根据代码类型构建不同的预览
    let previewContent = '';
    const code = props.code || '';

    // 如果是HTML或包含HTML标签的内容，直接渲染
    if (code.includes('<html>') || code.includes('<body>')) {
        previewContent = code;
    } else if (code.includes('<template>') && code.includes('<script lang="ts">')) {
        // Vue单文件组件，用注释提示
        previewContent = `
      <h2>Vue 组件预览</h2>
      <p>这是一个 Vue 单文件组件，需要在 Vue 项目中使用。</p>
      <pre><code>${code.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</code></pre>
    `;
    } else if (
        code.includes('@media') ||
        code.includes('@keyframes') ||
        (code.includes('{') && code.includes('}') && !code.includes('function'))
    ) {
        // CSS预览
        previewContent = `
      <style>${code}</style>
      <div class="css-preview">
        <h2>CSS预览</h2>
        <p>CSS已加载到此页面。请检查应用的样式。</p>
        <div class="demo-elements">
          <button class="btn">按钮</button>
          <div class="box">样式盒子</div>
          <p class="text">文本样例</p>
        </div>
      </div>
    `;
    } else {
        // 其他类型的代码，显示格式化的代码
        previewContent = `
      <h2>代码预览</h2>
      <pre><code>${code.replace(/</g, '&lt;').replace(/>/g, '&gt;')}</code></pre>
    `;
    }

    return `
    <!DOCTYPE html>
    <html>
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
          body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
            padding: 20px;
            margin: 0;
            line-height: 1.6;
          }
          pre {
            background-color: var(--color-input-bg);
            padding: 15px;
            border-radius: 4px;
            overflow: auto;
            font-size: var(--app-font-14);
          }
          code {
            font-family: 'SF Mono', Menlo, Monaco, Consolas, monospace;
          }
          .demo-elements {
            margin-top: 20px;
            display: flex;
            flex-direction: column;
            gap: 15px;
          }
          .btn {
            padding: 8px 16px;
            border: 1px solid #ccc;
            border-radius: 6px;
            cursor: pointer;
            width: fit-content;
            font-family: -apple-system, BlinkMacSystemFont, sans-serif;
          }
          .box {
            width: 100px;
            height: 100px;
            background: var(--color-window-titlebar-border);
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: 8px;
          }
        </style>
      </head>
      <body>
        ${previewContent}
      </body>
    </html>
  `;
});
</script>

<style scoped>
/* 预览HTML内部的样式保持不变，因为它是动态生成的HTML内容 */
</style>
