<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.filePlaceholder') }}</label>
                <el-input v-model="inputPath" :placeholder="t('toolbox.filePlaceholder')" />
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.outputDirPlaceholder') }}</label>
                <el-input v-model="outputDir" :placeholder="t('toolbox.outputDirPlaceholder')" />
            </div>
            <div class="form-group form-row-inline">
                <label class="form-label">{{ t('toolbox.splitParts') }}</label>
                <el-input-number v-model="parts" :min="2" :max="100" size="default" />
            </div>
            <div class="form-group">
                <el-button type="primary" @click="run">{{ t('toolbox.run') }}</el-button>
            </div>
            <p v-if="message" class="result-msg">{{ message }}</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useCsvSplit } from '../composables/tools/useCsvSplit';

const { t } = useI18n();
const { inputPath, outputDir, parts, message, run } = useCsvSplit();
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

.form-row-inline {
    display: flex;
    align-items: center;
    gap: 12px;
}

.form-row-inline .form-label {
    margin-bottom: 0;
    white-space: nowrap;
}

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}
</style>
