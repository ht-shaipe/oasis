<template>
    <div class="tool-panel">
        <div class="panel-body">
            <div class="form-group">
                <label class="form-label">{{ t('toolbox.dirPlaceholder') }}</label>
                <el-input v-model="dir" :placeholder="t('toolbox.dirPlaceholder')">
                    <template #append>
                        <el-button @click="run" :loading="loading" type="primary">
                            {{ t('toolbox.run') }}
                        </el-button>
                    </template>
                </el-input>
            </div>
            <div v-if="result" class="result-area">
                <div class="result-summary">
                    <span class="result-badge">
                        {{ t('toolbox.totalLines') }}: <strong>{{ result.total }}</strong>
                    </span>
                </div>
                <el-table :data="result.entries" max-height="300" size="small" stripe>
                    <el-table-column prop="path" :label="t('toolbox.filePath')" show-overflow-tooltip />
                    <el-table-column prop="lines" :label="t('toolbox.lines')" width="100" align="center" />
                </el-table>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useCsvStats } from '../composables/tools/useCsvStats';

const { t } = useI18n();
const { dir, loading, result, run } = useCsvStats();
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
