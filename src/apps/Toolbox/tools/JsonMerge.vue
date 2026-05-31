<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.inputDirPlaceholder') }}</label>
                <el-input v-model="inputDir" :placeholder="t('toolbox.inputDirPlaceholder')" />
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.outputFilePlaceholder') }}</label>
                <el-input v-model="outputPath" :placeholder="t('toolbox.outputFilePlaceholder')" />
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.jsonPathPlaceholder') }}</label>
                <el-input v-model="jsonPath" :placeholder="t('toolbox.jsonPathPlaceholder')" />
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
import { useJsonMerge } from '../composables/tools/useJsonMerge';

const { t } = useI18n();
const { inputDir, outputPath, jsonPath, message, run } = useJsonMerge();
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

.result-msg {
    color: #2d8c3c;
    margin-top: 12px;
    font-size: 13px;
    font-weight: 500;
}
</style>
