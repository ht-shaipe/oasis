import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';

export function useJsonMerge() {
    const { t } = useI18n();
    const inputDir = ref('');
    const outputPath = ref('');
    const jsonPath = ref('');
    const message = ref('');

    async function run() {
        if (!inputDir.value || !outputPath.value) {
            ElMessage.warning(t('toolbox.pathRequired'));
            return;
        }

        try {
            message.value = await invoke<string>('json_merge_files', {
                inputDir: inputDir.value,
                outputPath: outputPath.value,
                jsonPath: jsonPath.value,
            });
        } catch (error: any) {
            message.value = '';
            ElMessage.error(error);
        }
    }

    return {
        inputDir,
        outputPath,
        jsonPath,
        message,
        run,
    };
}
