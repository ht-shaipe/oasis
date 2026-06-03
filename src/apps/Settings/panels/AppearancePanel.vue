<template>
    <div class="section-panel">
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
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useThemeStore } from '@/store/theme';
import { useFontSizeStore, type FontSize } from '@/store/fontSize';

const { t } = useI18n();

const themeStore = useThemeStore();
const isDark = computed(() => themeStore.isDark);
const toggleTheme = () => themeStore.toggle();

const fontSizeStore = useFontSizeStore();
const fontSize = computed(() => fontSizeStore.size);
const fontSizeOptions = computed(() => [
    { value: 'small', label: t('settings.appearance.fontSizeSmall'), preview: 'Aa' },
    { value: 'medium', label: t('settings.appearance.fontSizeMedium'), preview: 'Aa' },
    { value: 'large', label: t('settings.appearance.fontSizeLarge'), preview: 'Aa' }
]);
const setFontSize = (size: FontSize) => fontSizeStore.setSize(size);
</script>

<style scoped>
.setting-row {
    padding-bottom: 16px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--color-card-border);
}

.setting-row:last-child {
    padding-bottom: 0;
    margin-bottom: 0;
    border-bottom: none;
}
</style>
