<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-row-split">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.ipRangePlaceholder') }}</label>
                    <el-input v-model="ipRange" :placeholder="t('toolbox.ipRangePlaceholder')" />
                </div>
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.portsPlaceholder') }}</label>
                    <el-input v-model="ports" :placeholder="t('toolbox.portsPlaceholder')" />
                </div>
            </div>
            <div class="form-row-split">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.timeout') }}</label>
                    <div class="inline-controls">
                        <el-input-number v-model="timeout" :min="100" :max="10000" :step="100" size="default" />
                        <span class="unit-text">ms</span>
                    </div>
                </div>
                <div class="form-group">
                    <label class="form-label">&nbsp;</label>
                    <el-checkbox v-model="showClosed">{{ t('toolbox.showClosed') }}</el-checkbox>
                </div>
            </div>
            <div class="form-group">
                <el-button type="primary" @click="run" :loading="loading">{{ t('toolbox.startScan') }}</el-button>
            </div>
            <div v-if="result" class="result-area">
                <pre class="scan-result-text">{{ result.format_text }}</pre>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useNetworkScan } from '../composables/tools/useNetworkScan';

const { t } = useI18n();
const { ipRange, ports, timeout, showClosed, loading, result, run } = useNetworkScan();
</script>

<style scoped>
.panel-body {
    display: flex;
    flex-direction: column;
    gap: 0;
}

.form-group {
    margin-bottom: 16px;
}

.form-label {
    display: block;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-tertiary, #999);
    margin-bottom: 6px;
    letter-spacing: 0.3px;
}

.form-row-split {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
}

.form-row-split .form-group {
    margin-bottom: 0;
}

.inline-controls {
    display: flex;
    align-items: center;
    gap: 6px;
}

.unit-text {
    font-size: 12px;
    color: var(--color-text-tertiary, #999);
}

.result-area {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-card-bg, rgba(255, 255, 255, 0.6));
    border-radius: 8px;
    border: 1px solid var(--color-card-border, rgba(0, 0, 0, 0.06));
}

.scan-result-text {
    background: var(--color-input-bg, #fff);
    padding: 14px;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.6;
    overflow-x: auto;
    white-space: pre;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    color: var(--color-text-primary, #333);
    border: 1px solid var(--color-card-border, rgba(0, 0, 0, 0.06));
}
</style>
