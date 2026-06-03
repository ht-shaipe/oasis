<template>
    <div class="section-panel">
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
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
</script>
