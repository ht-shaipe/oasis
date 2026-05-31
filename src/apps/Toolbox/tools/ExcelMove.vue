<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.excelFilePlaceholder') }}</label>
                <el-input v-model="excelPath" :placeholder="t('toolbox.excelFilePlaceholder')" />
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.colHeaderPlaceholder') }}</label>
                <el-input v-model="colHeader" :placeholder="t('toolbox.colHeaderPlaceholder')" />
            </div>
            <div class="form-row-split">
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.inputDirPlaceholder') }}</label>
                    <el-input v-model="inputDir" :placeholder="t('toolbox.inputDirPlaceholder')" />
                </div>
                <div class="form-group">
                    <label class="form-label">{{ t('toolbox.outputDirPlaceholder') }}</label>
                    <el-input v-model="outputDir" :placeholder="t('toolbox.outputDirPlaceholder')" />
                </div>
            </div>
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.suffixesPlaceholder') }}</label>
                <el-input v-model="suffixes" :placeholder="t('toolbox.suffixesPlaceholder')" />
            </div>
            <div class="form-group">
                <el-button @click="runPreview" :loading="loading">{{ t('toolbox.preview') }}</el-button>
                <el-button type="primary" @click="apply" :disabled="!hasPlan">{{ t('toolbox.apply') }}</el-button>
            </div>
            <div v-if="preview" class="result-area">
                <div class="result-summary">
                    <span class="result-badge">
                        {{ t('toolbox.matchResult') }}: <strong>{{ preview.found }}/{{ preview.total }}</strong>
                    </span>
                    <span class="result-badge">
                        {{ t('toolbox.missing') }}: <strong>{{ preview.missing }}</strong>
                    </span>
                    <span class="result-badge">
                        {{ t('toolbox.duplicate') }}: <strong>{{ preview.duplicate }}</strong>
                    </span>
                </div>
                <el-table :data="preview.items" max-height="300" size="small" stripe>
                    <el-table-column prop="status" :label="t('toolbox.status')" width="80" align="center" />
                    <el-table-column prop="file_name" :label="t('toolbox.fileName')" show-overflow-tooltip />
                    <el-table-column prop="base" :label="t('toolbox.keyword')" show-overflow-tooltip />
                </el-table>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useExcelMove } from '../composables/tools/useExcelMove';

const { t } = useI18n();
const { excelPath, colHeader, inputDir, outputDir, suffixes, loading, preview, hasPlan, runPreview, apply } = useExcelMove();
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

.result-area {
    margin-top: 16px;
    padding: 16px;
    background: var(--color-card-bg, rgba(255, 255, 255, 0.6));
    border-radius: 8px;
    border: 1px solid var(--color-card-border, rgba(0, 0, 0, 0.06));
}

.result-summary {
    display: flex;
    gap: 16px;
    margin-bottom: 12px;
    flex-wrap: wrap;
}

.result-badge {
    font-size: 13px;
    color: var(--color-text-primary, #333);
}

.result-badge strong {
    color: var(--color-menu-hover, #0078d7);
}
</style>
