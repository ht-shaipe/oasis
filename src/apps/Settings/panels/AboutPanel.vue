<template>
    <div class="section-panel">
        <h2 class="section-heading">{{ t('settings.about.title') }}</h2>
        <div class="about-card">
            <img src="/assets/icons/AppStore.svg" alt="Oasis" class="about-logo" />
            <div class="about-info">
                <h3>Oasis</h3>
                <span class="about-version">{{ t('about.version') }} {{ currentVersion }}</span>
                <span class="about-desc">{{ t('about.description') }}</span>
            </div>
        </div>

        <div class="update-section">
            <div class="update-row">
                <div class="update-info">
                    <span class="update-label">{{ t('update.title') }}</span>
                    <span class="update-status" :class="{ 'has-update': updateResult?.has_update }">
                        <template v-if="checking">{{ t('update.checkingForUpdate') }}</template>
                        <template v-else-if="updateResult?.has_update">
                            Oasis {{ updateResult.latest_version }} {{ t('update.published') }}
                        </template>
                        <template v-else-if="updateResult && !updateResult.has_update">
                            {{ t('update.upToDate') }}
                        </template>
                    </span>
                </div>
                <el-button
                    size="small"
                    :loading="checking"
                    @click="handleCheckUpdate"
                    class="check-update-btn">
                    {{ checking ? '' : t('update.title') }}
                </el-button>
            </div>
        </div>

        <div class="about-copyright">
            © 2026 <a href="https://htui.tech/" target="_blank">HongTui</a>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import type { CheckUpdateResult } from '@/composables/useAppUpdate';

const { t } = useI18n();

const currentVersion = ref('0.1.0');
const checking = ref(false);
const updateResult = ref<CheckUpdateResult | null>(null);

const getCurrentVersion = async () => {
    try {
        const result = await invoke<CheckUpdateResult>('check_update');
        currentVersion.value = result.current_version;
    } catch {
        currentVersion.value = '0.1.0';
    }
};

getCurrentVersion();

const handleCheckUpdate = async () => {
    checking.value = true;
    try {
        const result = await invoke<CheckUpdateResult>('check_update');
        updateResult.value = result;
        currentVersion.value = result.current_version;
        if (result.has_update) {
            ElMessage.info({
                message: t('update.newVersionAvailable') + ': Oasis ' + result.latest_version,
                duration: 5000,
            });
        } else {
            ElMessage.success(t('update.upToDate'));
        }
    } catch (e) {
        ElMessage.error(t('update.checkFailed'));
    } finally {
        checking.value = false;
    }
};
</script>

<style scoped>
.update-section {
    margin-top: 20px;
    padding: 16px 20px;
    background: var(--color-card-bg);
    border: 1px solid var(--color-card-border);
    border-radius: 10px;
}

.update-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.update-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.update-label {
    font-size: var(--app-font-13);
    font-weight: 500;
    color: var(--color-text-primary);
}

.update-status {
    font-size: var(--app-font-12);
    color: var(--color-text-tertiary);
}

.update-status.has-update {
    color: #42b883;
}

.check-update-btn {
    flex-shrink: 0;
}
</style>
