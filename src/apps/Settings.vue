<template>
    <MacWindow
        :title="t('settings.title')"
        :isMinimized="isMinimized"
        @close="closeApp"
        @minimize="toggleMinimize"
        width="780"
        height="520">
        <div class="settings-container">
            <!-- Left Sidebar: icon list -->
            <div class="settings-sidebar">
                <div
                    v-for="item in sidebarItems"
                    :key="item.id"
                    class="sidebar-item"
                    :class="{ active: activeSection === item.id }"
                    @click="activeSection = item.id"
                >
                    <img :src="item.icon" :alt="item.label" class="sidebar-icon" />
                    <span class="sidebar-label">{{ item.label }}</span>
                </div>
            </div>

            <!-- Right Content -->
            <div class="settings-content">
                <!-- General -->
                <div v-if="activeSection === 'general'" class="section-panel">
                    <h2 class="section-heading">{{ t('settings.general.title') }}</h2>

                    <div class="setting-row">
                        <div class="setting-info">
                            <span class="setting-label">{{ t('settings.workspace.title') }}</span>
                            <span class="setting-desc">{{ t('settings.workspace.desc') }}</span>
                        </div>
                    </div>
                    <div class="setting-row">
                        <div class="workspace-path-row">
                            <input
                                class="workspace-input"
                                :value="workspaceDir"
                                readonly
                                :placeholder="t('settings.workspace.placeholder')"
                            />
                            <button class="workspace-browse-btn" @click="pickDirectory" :disabled="pickingDir">
                                {{ pickingDir ? '...' : t('settings.workspace.browse') }}
                            </button>
                        </div>
                        <p v-if="workspaceStatus" class="workspace-status" :class="{ error: workspaceError }">
                            {{ workspaceStatus }}
                        </p>
                    </div>
                </div>

                <!-- Appearance -->
                <div v-if="activeSection === 'appearance'" class="section-panel">
                    <h2 class="section-heading">{{ t('settings.appearance.title') }}</h2>

                    <div class="setting-row">
                        <div class="setting-info">
                            <span class="setting-label">{{ t('settings.appearance.fontSize') }}</span>
                            <span class="setting-desc">{{ t('settings.appearance.fontSizeDesc') }}</span>
                        </div>
                        <div class="font-size-selector">
                            <button
                                v-for="option in fontSizeOptions"
                                :key="option.value"
                                class="font-size-option"
                                :class="{ active: fontSize === option.value }"
                                @click="setFontSize(option.value)"
                            >
                                <span class="font-size-label">{{ option.label }}</span>
                                <span class="font-size-preview">{{ option.preview }}</span>
                            </button>
                        </div>
                    </div>

                    <div class="setting-row">
                        <div class="setting-info">
                            <span class="setting-label">{{ t('settings.appearance.darkMode') }}</span>
                            <span class="setting-desc">{{ t('settings.appearance.darkModeDesc') }}</span>
                        </div>
                        <label class="toggle-switch">
                            <input type="checkbox" :checked="isDark" @change="toggleTheme" />
                            <span class="toggle-slider"></span>
                        </label>
                    </div>
                </div>

                <!-- About -->
                <div v-if="activeSection === 'about'" class="section-panel">
                    <h2 class="section-heading">{{ t('settings.about.title') }}</h2>
                    <div class="about-card">
                        <img src="/assets/icons/AppStore.svg" alt="Oasis" class="about-logo" />
                        <div class="about-info">
                            <h3>Oasis</h3>
                            <span class="about-version">{{ t('about.version') }} 1.0.0</span>
                            <span class="about-desc">{{ t('about.description') }}</span>
                        </div>
                    </div>
                    <div class="about-copyright">
                        © 2026 <a href="https://htui.tech/" target="_blank">HongTui</a>
                    </div>
                </div>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import MacWindow from '@/components/common/MacWindow.vue';
import { useI18n } from 'vue-i18n';
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useThemeStore } from '@/store/theme';
import { useFontSizeStore, type FontSize } from '@/store/fontSize';

const { t } = useI18n();

defineProps({
    isMinimized: {
        type: Boolean,
        default: false,
    },
});

const emit = defineEmits(['close', 'minimize']);

// Theme
const themeStore = useThemeStore();
const isDark = computed(() => themeStore.isDark);
const toggleTheme = () => themeStore.toggle();

// Font size
const fontSizeStore = useFontSizeStore();
const fontSize = computed(() => fontSizeStore.size);
const fontSizeOptions = computed(() => [
    { value: 'small', label: t('settings.appearance.fontSizeSmall'), preview: 'Aa' },
    { value: 'medium', label: t('settings.appearance.fontSizeMedium'), preview: 'Aa' },
    { value: 'large', label: t('settings.appearance.fontSizeLarge'), preview: 'Aa' }
]);
const setFontSize = (size: FontSize) => fontSizeStore.setSize(size);

// Active section
const activeSection = ref('general');

// Sidebar items
const sidebarItems = computed(() => [
    { id: 'general', icon: '/assets/icons/Settings.svg', label: t('settings.general.title') },
    { id: 'appearance', icon: '/assets/icons/Features.svg', label: t('settings.appearance.title') },
    { id: 'about', icon: '/assets/icons/AppStore.svg', label: t('settings.about.title') },
]);

// Workspace
const workspaceDir = ref('');
const pickingDir = ref(false);
const workspaceStatus = ref('');
const workspaceError = ref(false);

const loadWorkspaceDir = async () => {
    try {
        workspaceDir.value = await invoke<string>('get_workspace_dir');
        workspaceStatus.value = '';
        workspaceError.value = false;
    } catch (e) {
        workspaceStatus.value = String(e);
        workspaceError.value = true;
    }
};

const pickDirectory = async () => {
    pickingDir.value = true;
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: t('settings.workspace.selectTitle'),
        });
        if (selected) {
            const path = typeof selected === 'string' ? selected : selected;
            const result = await invoke<string>('set_workspace_dir', { path });
            workspaceDir.value = result;
            workspaceStatus.value = t('settings.workspace.saved');
            workspaceError.value = false;
            setTimeout(() => { workspaceStatus.value = ''; }, 3000);
        }
    } catch (e) {
        workspaceStatus.value = String(e);
        workspaceError.value = true;
    } finally {
        pickingDir.value = false;
    }
};

onMounted(() => {
    loadWorkspaceDir();
});

const closeApp = () => {
    emit('close');
};

const toggleMinimize = () => {
    emit('minimize');
};
</script>

<style scoped>
.settings-container {
    display: flex;
    height: 100%;
}

/* ── Sidebar ── */
.settings-sidebar {
    width: 200px;
    min-width: 200px;
    background: var(--color-sidebar-bg);
    border-right: 1px solid var(--color-sidebar-border);
    padding: 12px 0;
    overflow-y: auto;
}

.sidebar-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    cursor: pointer;
    transition: background 0.15s;
    border-radius: 6px;
    margin: 2px 8px;
}

.sidebar-item:hover {
    background: var(--color-sidebar-item-hover);
}

.sidebar-item.active {
    background: var(--color-sidebar-item-active);
}

.sidebar-icon {
    width: 28px;
    height: 28px;
    object-fit: contain;
    border-radius: 6px;
}

.sidebar-label {
    font-size: var(--app-font-13);
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* ── Content ── */
.settings-content {
    flex: 1;
    padding: 24px 28px;
    overflow-y: auto;
}

.section-panel {
    animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

.section-heading {
    font-size: var(--app-font-22);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 20px 0;
}

/* ── Setting Row ── */
.setting-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 16px;
}

.setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    margin-right: 16px;
}

.setting-label {
    font-size: var(--app-font-13);
    font-weight: 500;
    color: var(--color-text-primary);
}

.setting-desc {
    font-size: var(--app-font-12);
    color: var(--color-text-tertiary);
    line-height: 1.4;
}

/* ── Workspace Path ── */
.workspace-path-row {
    display: flex;
    gap: 8px;
    width: 100%;
}

.workspace-input {
    flex: 1;
    padding: 6px 10px;
    font-size: var(--app-font-12);
    font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
    background: var(--color-input-bg);
    border: 1px solid var(--color-input-border);
    border-radius: 6px;
    color: var(--color-text-secondary);
    outline: none;
}

.workspace-input:focus {
    border-color: var(--code-accent);
}

.workspace-browse-btn {
    padding: 6px 14px;
    font-size: var(--app-font-12);
    border-radius: 6px;
    border: 1px solid var(--color-input-border);
    background: var(--color-input-bg);
    color: var(--color-text-primary);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s;
}

.workspace-browse-btn:hover {
    background: var(--color-sidebar-item-hover);
}

.workspace-browse-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.workspace-status {
    margin: 4px 0 0 0;
    font-size: var(--app-font-11);
    color: #34c759;
}

.workspace-status.error {
    color: #ff3b30;
}

/* ── Toggle Switch (macOS style) ── */
.toggle-switch {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 22px;
    flex-shrink: 0;
    margin-top: 2px;
}

.toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
}

.toggle-slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: #ccc;
    border-radius: 22px;
    transition: background 0.25s;
}

.toggle-slider::before {
    content: '';
    position: absolute;
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background: white;
    border-radius: 50%;
    transition: transform 0.25s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.15);
}

.toggle-switch input:checked + .toggle-slider {
    background: #34c759;
}

.toggle-switch input:checked + .toggle-slider::before {
    transform: translateX(18px);
}

/* ── Font Size Selector ── */
.font-size-selector {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
}

.font-size-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 52px;
    border-radius: 8px;
    border: 1.5px solid var(--color-input-border);
    background: var(--color-input-bg);
    cursor: pointer;
    transition: all 0.15s;
    padding: 4px;
    gap: 2px;
}

.font-size-option:hover {
    border-color: var(--code-accent);
    background: var(--color-sidebar-item-hover);
}

.font-size-option.active {
    border-color: #1977f3;
    background: rgba(25, 119, 243, 0.08);
    box-shadow: 0 0 0 1px rgba(25, 119, 243, 0.2);
}

.font-size-label {
    font-size: var(--app-font-11);
    color: var(--color-text-secondary);
    font-weight: 500;
}

.font-size-option.active .font-size-label {
    color: #1977f3;
}

.font-size-preview {
    font-weight: 600;
    color: var(--color-text-primary);
    transition: font-size 0.15s;
}

.font-size-option:first-child .font-size-preview {
    font-size: var(--app-font-13);
}

.font-size-option:nth-child(2) .font-size-preview {
    font-size: var(--app-font-16);
}

.font-size-option:last-child .font-size-preview {
    font-size: var(--app-font-20);
}

/* ── About ── */
.about-card {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
    background: var(--color-card-bg);
    border: 1px solid var(--color-card-border);
    border-radius: 10px;
}

.about-logo {
    width: 64px;
    height: 64px;
    object-fit: contain;
}

.about-info h3 {
    margin: 0;
    font-size: var(--app-font-18);
    font-weight: 600;
    color: var(--color-text-primary);
}

.about-version {
    display: block;
    font-size: var(--app-font-12);
    color: var(--color-text-tertiary);
    margin-top: 2px;
}

.about-desc {
    display: block;
    font-size: var(--app-font-13);
    color: var(--color-text-secondary);
    margin-top: 6px;
}

.about-copyright {
    margin-top: 20px;
    font-size: var(--app-font-12);
    color: var(--color-text-tertiary);
}

.about-copyright a {
    color: var(--color-link);
    text-decoration: none;
}

.about-copyright a:hover {
    color: var(--color-link-hover);
}
</style>
