<template>
    <!-- 使用登录组件 -->
    <LoginScreen v-if="isLoading" :isResourcesLoaded="resourcesLoaded" @login-complete="handleLoginComplete" />

    <!-- 主桌面界面 -->
    <div
        class="mac-desktop"
        @mousedown.left="closeAllMenus"
        @contextmenu.prevent="handleContextMenu"
        @mousedown="handleDesktopMouseDown">
        <!-- Mac顶部导航栏 -->
        <MenuBar />

        <!-- Mac桌面背景和图标 -->
        <DesktopIcons :apps="apps" @openApp="openApp" />

        <!-- 底部Dock栏 -->
        <Dock :apps="apps" @openApp="openApp" />

        <!-- 右键菜单 -->
        <ContextMenu :visible="showContextMenu" :position="contextMenuPosition" @close="showContextMenu = false" />

        <!-- 动态渲染应用窗口 -->
        <Teleport v-for="app in apps" :key="app.id" to="body">
            <component
                :is="app.component"
                v-if="windowStates[app.id].show"
                :isMinimized="windowStates[app.id].isMinimized"
                v-bind="getAppProps(app.id)"
                @close="closeApp(app.id)"
                @minimize="toggleMinimize(app.id)"
                @updateGeneratedCode="updateGeneratedCode"
                @updateSessionInfo="updateSessionInfo"
                @openApp="openApp"
                @codeUpdated="handleCodeUpdated"
                @loadVersion="handleLoadVersion"
                @continueVersion="handleContinueVersion"
                @codeGenerated="handleContinueCodeGenerated" />
        </Teleport>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { apps } from '@/config/apps';

// 导入系统组件
import LoginScreen from '@/components/system/LoginScreen.vue';
import MenuBar from '@/components/system/MenuBar.vue';
import Dock from '@/components/system/Dock.vue';
import DesktopIcons from '@/components/system/DesktopIcons.vue';
import ContextMenu from '@/components/system/ContextMenu.vue';

// 加载状态
const isLoading = ref(true);
const resourcesLoaded = ref(false);

// 窗口状态统一管理
const windowStates = reactive<Record<string, { show: boolean; isMinimized: boolean }>>(
    apps.reduce((acc, app) => {
        acc[app.id] = { show: false, isMinimized: false };
        return acc;
    }, {} as any),
);

const appWindow = getCurrentWindow();

// 桌面壁纸
const currentWallpaper = ref(1);

// 右键菜单状态
const showContextMenu = ref(false);
const contextMenuPosition = ref({ x: 0, y: 0 });

// 生成的代码和相关数据
const generatedCode = ref('');
const originalPrompt = ref('');
const currentprojectId = ref('');
const currentVersionId = ref('');
const filename = ref('');

// 获取应用对应的 Props
const getAppProps = (appId: string) => {
    switch (appId) {
        case 'editor':
            return { code: generatedCode.value, filename: filename.value };
        case 'safari':
            return { code: generatedCode.value };
        case 'Finder':
            return { currentprojectId: currentprojectId.value };
        case 'continue-dialog':
            return {
                projectId: currentprojectId.value,
                versionId: currentVersionId.value,
                originalPrompt: originalPrompt.value,
                currentCode: generatedCode.value,
            };
        default:
            return {};
    }
};

// 关闭应用
const closeApp = (appId: string) => {
    windowStates[appId].show = false;
};

// 切换最小化状态
const toggleMinimize = (appId: string) => {
    windowStates[appId].isMinimized = !windowStates[appId].isMinimized;
};

// 显示右键菜单
const handleContextMenu = (event: MouseEvent) => {
    // 阻止默认的浏览器右键菜单
    event.preventDefault();

    // 设置菜单位置
    contextMenuPosition.value = {
        x: event.clientX,
        y: event.clientY,
    };

    // 显示自定义菜单
    showContextMenu.value = true;
};

// 点击桌面时关闭所有菜单
const closeAllMenus = () => {
    showContextMenu.value = false;
};

const handleDesktopMouseDown = (event: MouseEvent) => {
    if (event.button !== 0 || event.target !== event.currentTarget) {
        return;
    }

    void appWindow.startDragging();
};

// 打开应用
const openApp = (appName: string | { type: string; url: string; target: string }) => {
    // 检查是否是带有URL的对象格式
    if (typeof appName === 'object' && appName.type === 'safari') {
        if (appName.target === '_blank') {
            window.open(appName.url, '_blank');
        } else {
            windowStates.safari.show = true;
            windowStates.safari.isMinimized = false;
            localStorage.setItem('safariUrl', appName.url || '');
        }
    } else {
        const appId = appName as string;
        if (windowStates[appId]) {
            windowStates[appId].show = true;
            windowStates[appId].isMinimized = false;
            if (appId === 'safari') {
                localStorage.removeItem('safariUrl');
            }
        }
    }
};

// 更新生成的代码
const updateGeneratedCode = (code: string) => {
    generatedCode.value = code;
    filename.value = 'index.html';
    // 确保编辑器窗口已打开
    if (!windowStates.editor.show) {
        windowStates.editor.show = true;
        windowStates.editor.isMinimized = false;
    }
};

// 更新会话信息
const updateSessionInfo = (prompt: string, projectId: string, versionId: string, shouldOpenContinueDialog = false) => {
    originalPrompt.value = prompt || '';
    currentprojectId.value = projectId || '';
    currentVersionId.value = versionId || '';

    // 打开相关窗口的逻辑
    if (!windowStates.safari.show && generatedCode.value && !generatedCode.value.startsWith('// 代码正在生成中')) {
        // 延迟打开预览窗口
        setTimeout(() => {
            openApp('safari');
        }, 800);
    }

    // 根据请求决定是否打开继续对话组件
    if (shouldOpenContinueDialog && projectId && !windowStates['continue-dialog'].show) {
        setTimeout(() => {
            windowStates['continue-dialog'].show = true;
            windowStates['continue-dialog'].isMinimized = false;
        }, 1000);
    }
};

// 处理代码更新事件
const handleCodeUpdated = (code: string) => {
    generatedCode.value = code;
};

// 处理版本加载事件
const handleLoadVersion = (versionDetail: { code?: string; prompt?: string } | null) => {
    if (versionDetail && versionDetail.code) {
        generatedCode.value = versionDetail.code;
        originalPrompt.value = versionDetail.prompt || '';

        // 打开编辑器和预览
        windowStates.editor.show = true;
        windowStates.editor.isMinimized = false;

        setTimeout(() => {
            windowStates.safari.show = true;
            windowStates.safari.isMinimized = false;
        }, 500);
    }
};

// 处理继续对话版本
const handleContinueVersion = (version: { code?: string; prompt?: string } | null) => {
    if (version) {
        // 先加载这个版本
        handleLoadVersion(version);

        // 打开继续对话窗口
        setTimeout(() => {
            windowStates['continue-dialog'].show = true;
            windowStates['continue-dialog'].isMinimized = false;
        }, 800);
    }
};

// 处理继续生成的代码
const handleContinueCodeGenerated = async (code: string, _additionalPrompt: string) => {
    // 更新代码
    generatedCode.value = code;

    // 确保编辑器和预览窗口打开并更新
    windowStates.editor.show = true;
    windowStates.editor.isMinimized = false;

    setTimeout(() => {
        windowStates.safari.show = true;
        windowStates.safari.isMinimized = false;
    }, 500);

    // 如果版本管理器打开，刷新它
    if (windowStates.Finder.show) {
        // 通过重新打开来刷新
        windowStates.Finder.show = false;
        setTimeout(() => {
            windowStates.Finder.show = true;
        }, 100);
    }
};

// 处理登录完成事件
const handleLoginComplete = () => {
    isLoading.value = false;
};

// 检查资源加载
const checkResourcesLoaded = () => {
    return new Promise((resolve) => {
        if (document.readyState === 'complete') {
            resolve(true);
        } else {
            window.addEventListener('load', () => {
                resolve(true);
            });
        }
    });
};

// 页面加载完成后执行
onMounted(async () => {
    // 检查资源加载状态
    checkResourcesLoaded().then(() => {
        resourcesLoaded.value = true;
    });

    // 从本地存储加载保存的壁纸
    const savedWallpaper = localStorage.getItem('wallpaper') || '1';
    currentWallpaper.value = parseInt(savedWallpaper);
    const desktop = document.querySelector('.mac-desktop');
    if (desktop) {
        const el = desktop as HTMLElement;
        const preloadImage = new Image();
        preloadImage.src = `/assets/wallpaper/${currentWallpaper.value}.jpg`;
        preloadImage.onload = () => {
            el.style.backgroundImage = `url('/assets/wallpaper/${currentWallpaper.value}.jpg')`;
            el.style.transition = 'background-image 1s ease-in-out';
        };
    }
});
</script>

<style scoped>
/* Mac桌面和全局样式 */
.mac-desktop {
    width: 100%;
    height: 100vh;
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    overflow: hidden;
    position: relative;
    user-select: none;
    display: flex;
    flex-direction: column;
    cursor: default;
    box-sizing: border-box;
    transition:
        background-image 1s ease-in-out,
        filter 0.3s ease;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
