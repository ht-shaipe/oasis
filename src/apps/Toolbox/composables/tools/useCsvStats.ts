import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';
import type { CsvStatsResult } from '../../types';

export function useCsvStats() {
    const { t } = useI18n();
    const dir = ref('');
    const loading = ref(false);
    const result = ref<CsvStatsResult | null>(null);

    async function run() {
        if (!dir.value) {
            ElMessage.warning(t('toolbox.dirRequired'));
            return;
        }

        loading.value = true;
        try {
            result.value = await invoke<CsvStatsResult>('csv_scan_dir', { dir: dir.value });
        } catch (error: any) {
            ElMessage.error(error);
            result.value = null;
        } finally {
            loading.value = false;
        }
    }

    return {
        dir,
        loading,
        result,
        run,
    };
}
