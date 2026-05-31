<template>
    <MacWindow :title="t('browser.title')" @close="emit('close')" @minimize="emit('minimize')">
        <div class="browser-container">
            <h3>{{ t('browser.cdpLauncher') }}</h3>
            <p class="desc">{{ t('browser.desc') }}</p>

            <div class="form-row">
                <el-button type="primary" @click="findChrome" :loading="findLoading">
                    {{ t('browser.findChrome') }}
                </el-button>
                <span v-if="chromePath" class="chrome-path">{{ chromePath }}</span>
            </div>

            <div class="form-row">
                <el-button type="success" @click="launchChrome" :disabled="!chromePath" :loading="launchLoading">
                    {{ t('browser.launchChrome') }}
                </el-button>
            </div>

            <div v-if="launchMsg" class="launch-msg">
                <p>{{ launchMsg }}</p>
                <p class="hint">{{ t('browser.hint') }}</p>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';

const { t } = useI18n();

const emit = defineEmits<{
    close: [];
    minimize: [];
}>();

const chromePath = ref('');
const findLoading = ref(false);
const launchLoading = ref(false);
const launchMsg = ref('');

async function findChrome() {
    findLoading.value = true;
    try {
        chromePath.value = await invoke<string>('find_chrome_path');
    } catch (e: any) {
        ElMessage.error(e);
    } finally {
        findLoading.value = false;
    }
}

async function launchChrome() {
    launchLoading.value = true;
    try {
        launchMsg.value = await invoke<string>('launch_chrome_cdp');
    } catch (e: any) {
        ElMessage.error(e);
    } finally {
        launchLoading.value = false;
    }
}

defineOptions({ name: 'BrowserApp' });
</script>

<style scoped>
.browser-container {
    padding: 20px;
    color: var(--color-text, #ccc);
}

h3 {
    margin: 0 0 8px;
    font-size: 16px;
}

.desc {
    font-size: 13px;
    color: var(--color-text-muted, #999);
    margin-bottom: 20px;
}

.form-row {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-bottom: 16px;
}

.chrome-path {
    font-size: 13px;
    font-family: monospace;
    color: var(--color-success, #67c23a);
}

.launch-msg {
    margin-top: 16px;
    font-size: 13px;
    color: var(--color-success, #67c23a);
}
.hint {
    color: var(--color-text-muted, #888);
    font-size: 12px;
}
</style>