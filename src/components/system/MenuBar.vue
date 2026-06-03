<template>
    <div
        class="mac-menubar"
        ref="menuBarRef"
        :class="{ 'mac-windowed': isMacPlatform && !isFullscreenMode }"
        @click.stop
        @mousedown="handleMenuBarMouseDown"
        data-tauri-drag-region>
        <div class="menubar-left">
            <div class="mac-logo" @click="toggleAppleMenu">
                <img src="/assets/logo.png" alt="Logo" />
                <!-- Apple Menu -->
                <div class="apple-menu" v-if="showAppleMenu">
                    <div class="menu-item">{{ t('menu.about') }}</div>
                    <div class="menu-divider"></div>
                    <div class="menu-item">{{ t('menu.preferences') }}...</div>
                    <div class="menu-item">App Store...</div>
                    <div class="menu-divider"></div>
                    <div class="menu-item">{{ t('menu.forceQuit') }}...</div>
                    <div class="menu-divider"></div>
                    <div class="menu-item">{{ t('menu.sleep') }}</div>
                    <div class="menu-item">{{ t('menu.restart') }}...</div>
                    <div class="menu-item">{{ t('menu.shutDown') }}...</div>
                </div>
            </div>
            <div class="menubar-items">
                <!-- <span class="menubar-item active">Oasis</span> -->
                <span class="menubar-item" @click="toggleFileMenu">
                    {{ t('menu.file') }}
                    <!-- File Menu -->
                    <div class="dropdown-menu" v-if="showFileMenu">
                        <div class="menu-item">{{ t('app.new') }}</div>
                        <div class="menu-item">{{ t('app.open') }}</div>
                        <div class="menu-item">{{ t('app.save') }}</div>
                        <div class="menu-divider"></div>
                        <div class="menu-item">{{ t('app.export') }}</div>
                    </div>
                </span>
                <span class="menubar-item">{{ t('menu.edit') }}</span>
                <span class="menubar-item">{{ t('menu.view') }}</span>
                <span class="menubar-item">{{ t('macWindow.close') }}</span>
                <span class="menubar-item">{{ t('menu.help') }}</span>
            </div>
        </div>
        <div class="menubar-right">
            <div class="menubar-icons">
                <span
                    v-if="menuBarConfig.rightVisible.notification"
                    class="menubar-icon"
                    @click="toggleNotificationCenter">
                    <el-icon><Bell /></el-icon>
                </span>
                <span v-if="menuBarConfig.rightVisible.clipboard" class="menubar-icon">
                    <el-icon><CopyDocument /></el-icon>
                </span>
                <span v-if="menuBarConfig.rightVisible.credits" class="menubar-icon" @click="toggleSignInModal">
                    <el-icon><Coin /></el-icon>
                </span>
                <span
                    v-if="menuBarConfig.rightVisible.theme"
                    class="menubar-icon theme-toggle"
                    @click="theme.toggle()"
                    :title="theme.isDark ? t('theme.switchToLight') : t('theme.switchToDark')">
                    <el-icon><Sunny v-if="theme.isDark" /><Moon v-else /></el-icon>
                </span>
                <span
                    v-if="menuBarConfig.rightVisible.locale"
                    class="menubar-icon locale-toggle"
                    @click="toggleLocale"
                    :title="localeStore.locale === 'zh-CN' ? 'English' : '中文'">
                    {{ localeStore.locale === 'zh-CN' ? 'EN' : '中' }}
                </span>
                <span v-if="menuBarConfig.rightVisible.battery" class="menubar-battery">
                    <div class="battery-icon">
                        <div class="battery-level"></div>
                    </div>
                    <span>100%</span>
                </span>
                <span v-if="menuBarConfig.rightVisible.clock" class="menubar-date-time" @click="toggleCalendar">
                    <span class="menubar-time">{{ currentTime }}</span>
                </span>
            </div>
        </div>
    </div>
    <!-- Sign-in Dialog -->
    <SignInModal v-model:visible="showSignInModal" />

    <!-- Notification Center -->
    <NotificationCenter v-if="showNotificationCenter" @close="showNotificationCenter = false" />

    <!-- Calendar -->
    <Calendar v-if="showCalendar" @close="showCalendar = false" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { CopyDocument, Bell, Coin, Sunny, Moon } from '@element-plus/icons-vue';
import SignInModal from './SignInModal.vue';
import NotificationCenter from './NotificationCenter.vue';
import Calendar from './Calendar.vue';
import { useThemeStore } from '@/store/theme';
import { useLocaleStore } from '@/store/locale';
import { menuBarConfig } from '@/config/menuBar';
import { useI18n } from 'vue-i18n';

const theme = useThemeStore();
const localeStore = useLocaleStore();
const { t } = useI18n();
const appWindow = getCurrentWindow();

const isMacPlatform = /Mac/i.test(navigator.userAgent);
const isFullscreenMode = ref(false);
const menuBarRef = ref<HTMLElement | null>(null);
let syncFrame: number | null = null;
let unlistenResize: (() => void) | null = null;
let unlistenScaleChange: (() => void) | null = null;

// 菜单状态
const showAppleMenu = ref(false);
const showFileMenu = ref(false);
const showSignInModal = ref(false);
const showNotificationCenter = ref(false);
const showCalendar = ref(false);

// 事件发射
const emit = defineEmits(['toggleNotificationCenter', 'toggleCalendar', 'updateCredits']);

// 切换Apple菜单
const toggleAppleMenu = () => {
    showAppleMenu.value = !showAppleMenu.value;
    showFileMenu.value = false;
};

// 切换文件菜单
const toggleFileMenu = () => {
    showFileMenu.value = !showFileMenu.value;
    showAppleMenu.value = false;
};

// 切换通知中心
const toggleNotificationCenter = () => {
    showNotificationCenter.value = !showNotificationCenter.value;
    showCalendar.value = false;
    showAppleMenu.value = false;
    showFileMenu.value = false;
};

// 切换日历
const toggleCalendar = () => {
    showCalendar.value = !showCalendar.value;
    showNotificationCenter.value = false;
    showAppleMenu.value = false;
    showFileMenu.value = false;
};

// 切换签到弹窗
const toggleSignInModal = () => {
    showSignInModal.value = true;
    showAppleMenu.value = false;
    showFileMenu.value = false;
};

// 切换语言
const toggleLocale = () => {
    localeStore.toggleLocale();
};

const closeMenuPanels = () => {
    showAppleMenu.value = false;
    showFileMenu.value = false;
    showNotificationCenter.value = false;
    showCalendar.value = false;
};

const handleDocumentMouseDown = (event: MouseEvent) => {
    const target = event.target as Node | null;
    if (!target) {
        return;
    }

    if (menuBarRef.value?.contains(target)) {
        return;
    }

    closeMenuPanels();
};

const syncWindowMode = async () => {
    const [fullscreen, maximized] = await Promise.all([appWindow.isFullscreen(), appWindow.isMaximized()]);
    isFullscreenMode.value = fullscreen || maximized;
};

const scheduleSyncWindowMode = () => {
    if (syncFrame !== null) {
        cancelAnimationFrame(syncFrame);
    }

    syncFrame = window.requestAnimationFrame(() => {
        syncFrame = null;
        void syncWindowMode();
    });
};

const handleMenuBarMouseDown = (event: MouseEvent) => {
    if (event.button !== 0 || event.target !== event.currentTarget) {
        return;
    }

    void appWindow.startDragging();
};

// 当前时间
const currentTime = ref('');
let timeInterval: ReturnType<typeof setInterval> | null = null;

// 更新时间的函数
const updateTime = () => {
    const now = new Date();
    const hours = now.getHours().toString().padStart(2, '0');
    const minutes = now.getMinutes().toString().padStart(2, '0');
    const seconds = now.getSeconds().toString().padStart(2, '0');
    currentTime.value = `${hours}:${minutes}:${seconds}`;
};

// 组件挂载后初始化
onMounted(() => {
    // 初始化时间并设置定时器
    updateTime();
    timeInterval = setInterval(updateTime, 1000);
    void syncWindowMode();
    void appWindow.onResized(scheduleSyncWindowMode).then((unlisten) => {
        unlistenResize = unlisten;
    });
    void appWindow.onScaleChanged(scheduleSyncWindowMode).then((unlisten) => {
        unlistenScaleChange = unlisten;
    });
    window.addEventListener('resize', scheduleSyncWindowMode);
    document.addEventListener('mousedown', handleDocumentMouseDown);
});

// 组件卸载前清理
onBeforeUnmount(() => {
    // 清除定时器
    if (timeInterval) {
        clearInterval(timeInterval);
        timeInterval = null;
    }

    if (syncFrame !== null) {
        cancelAnimationFrame(syncFrame);
        syncFrame = null;
    }

    unlistenResize?.();
    unlistenResize = null;

    unlistenScaleChange?.();
    unlistenScaleChange = null;

    window.removeEventListener('resize', scheduleSyncWindowMode);
    document.removeEventListener('mousedown', handleDocumentMouseDown);
});
</script>

<style scoped>
/* Mac顶部菜单栏 */
.mac-menubar {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 32px;
    /* background-color: var(--color-menubar-bg); */
    backdrop-filter: blur(10px);
    color: var(--color-menubar-text);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 10px;
    font-size: var(--app-font-13);
    z-index: var(--z-index-menu-bar);
    box-shadow: 0 1px 5px rgba(0, 0, 0, 0.1);
    box-sizing: border-box;
}

.mac-menubar.mac-windowed {
    padding-left: 74px;
    padding-right: 14px;
}

.menubar-left {
    display: flex;
    align-items: center;
}

.mac-logo {
    width: 18px;
    height: 18px;
    margin-right: 15px;
    position: relative;
    cursor: pointer;
    margin-left: 8px;
}

.mac-logo img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.menubar-items {
    display: flex;
    gap: 18px;
}

.menubar-item {
    cursor: pointer;
    opacity: 0.8;
    transition: opacity 0.2s;
    position: relative;
}

.menubar-item:hover,
.menubar-item.active {
    opacity: 1;
}

.menubar-right {
    display: flex;
    align-items: center;
    padding-right: 8px;
}

.menubar-icons {
    display: flex;
    align-items: center;
    gap: 12px;
}

.menubar-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0.8;
    transition: opacity 0.2s;
    height: 16px;
}

.menubar-icon:hover {
    opacity: 1;
}

.menubar-icon .el-icon {
    font-size: var(--app-font-14);
}

.menubar-date-time {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: var(--app-font-15);
}

.menubar-date {
    opacity: 0.7;
    font-size: var(--app-font-10);
}

.menubar-time {
    font-weight: 600;
}

.menubar-battery {
    display: flex;
    align-items: center;
    font-size: var(--app-font-12);
    gap: 4px;
}

.battery-icon {
    width: 20px;
    height: 10px;
    border: 1px solid white;
    border-radius: 2px;
    position: relative;
    display: flex;
    align-items: center;
    padding: 1px;
}

.battery-icon:after {
    content: '';
    position: absolute;
    right: -3px;
    top: 2px;
    height: 6px;
    width: 2px;
    background: white;
    border-radius: 0 1px 1px 0;
}

.battery-level {
    background-color: var(--color-menubar-text);
    height: 100%;
    width: 100%;
    border-radius: 1px;
}

/* 下拉菜单样式 */
.apple-menu {
    position: absolute;
    top: 25px;
    left: 0;
    background-color: var(--color-menu-bg);
    backdrop-filter: blur(10px);
    border-radius: 5px;
    box-shadow: 0 5px 20px var(--color-shadow);
    min-width: 200px;
    z-index: var(--z-index-dropdown);
    padding: 5px 0;
}

.dropdown-menu {
    position: absolute;
    top: 25px;
    left: 0;
    background-color: var(--color-menu-bg);
    backdrop-filter: blur(10px);
    border-radius: 5px;
    box-shadow: 0 5px 20px var(--color-shadow);
    min-width: 180px;
    z-index: var(--z-index-dropdown);
    padding: 5px 0;
}

.menu-item {
    padding: 5px 15px;
    cursor: pointer;
    font-size: var(--app-font-13);
    color: var(--color-menu-text);
}

.menu-item:hover {
    background-color: #0078d7;
}

.menu-divider {
    height: 1px;
    background-color: var(--color-menu-divider);
    margin: 5px 0;
}
</style>
