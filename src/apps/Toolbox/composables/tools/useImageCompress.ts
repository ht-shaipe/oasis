import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ElMessage } from 'element-plus';
import { useI18n } from 'vue-i18n';

export interface ImageInfo {
    path: string;
    name: string;
    size: number;
    width: number;
    height: number;
    format: string;
}

export interface CompressResultItem {
    path: string;
    name: string;
    original_size: number;
    compressed_size: number;
    ratio: number;
    success: boolean;
    error: string | null;
    width: number;
    height: number;
}

export type ImageStatus = 'pending' | 'compressing' | 'done' | 'error';

export interface ImageItem {
    path: string;
    name: string;
    originalSize: number;
    width: number;
    height: number;
    format: string;
    status: ImageStatus;
    compressedSize: number;
    compressedWidth: number;
    compressedHeight: number;
    ratio: number;
    errorMsg: string;
}

export function useImageCompress() {
    const { t } = useI18n();
    const images = ref<ImageItem[]>([]);
    const quality = ref(80);
    const format = ref<string | null>(null);
    const scale = ref<number | null>(null);
    const loading = ref(false);
    const totalOriginal = ref(0);
    const totalCompressed = ref(0);

    function toItems(list: ImageInfo[]): ImageItem[] {
        return list.map((img) => ({
            path: img.path,
            name: img.name,
            originalSize: img.size,
            width: img.width,
            height: img.height,
            format: img.format,
            status: 'pending' as ImageStatus,
            compressedSize: 0,
            compressedWidth: 0,
            compressedHeight: 0,
            ratio: 0,
            errorMsg: '',
        }));
    }

    async function scanDir(dirPath: string, recursive: boolean = false) {
        try {
            const list = await invoke<ImageInfo[]>('image_compress_scan_dir', {
                params: { dir_path: dirPath, recursive },
            });
            images.value = toItems(list);
        } catch (error: any) {
            ElMessage.error(error);
        }
    }

    async function scanFiles(filePaths: string[]) {
        try {
            const list = await invoke<ImageInfo[]>('image_compress_scan_files', {
                filePaths,
            });
            const existingPaths = new Set(images.value.map((i) => i.path));
            const newItems = toItems(list.filter((i) => !existingPaths.has(i.path)));
            images.value = [...images.value, ...newItems];
        } catch (error: any) {
            ElMessage.error(error);
        }
    }

    function removeImage(index: number) {
        images.value.splice(index, 1);
    }

    function clearImages() {
        images.value = [];
        totalOriginal.value = 0;
        totalCompressed.value = 0;
    }

    function findItemByPath(path: string): ImageItem | undefined {
        return images.value.find((i) => i.path === path);
    }

    async function compress() {
        if (images.value.length === 0) {
            ElMessage.warning(t('toolbox.imageCompress.noImages'));
            return;
        }

        loading.value = true;
        totalOriginal.value = 0;
        totalCompressed.value = 0;

        for (const item of images.value) {
            item.status = 'compressing';
            item.compressedSize = 0;
            item.ratio = 0;
            item.errorMsg = '';
        }

        let unlisten: UnlistenFn | null = null;

        try {
            unlisten = await listen<{ index: number; total: number; result: CompressResultItem }>('image-compress-progress', (event) => {
                const result = event.payload.result;
                const item = findItemByPath(result.path);
                if (!item) return;

                if (result.success) {
                    item.status = 'done';
                    item.compressedSize = result.compressed_size;
                    item.compressedWidth = result.width;
                    item.compressedHeight = result.height;
                    item.ratio = result.ratio;
                    totalOriginal.value += result.original_size;
                    totalCompressed.value += result.compressed_size;
                } else {
                    item.status = 'error';
                    item.errorMsg = result.error || 'Unknown error';
                    totalOriginal.value += result.original_size;
                }
            });

            const result = await invoke<{
                results: CompressResultItem[];
                total_original: number;
                total_compressed: number;
                total_ratio: number;
                success_count: number;
                fail_count: number;
            }>('image_compress_batch', {
                params: {
                    input_paths: images.value.map((i) => i.path),
                    output_dir: null,
                    quality: quality.value,
                    format: format.value,
                    scale: scale.value,
                },
            });

            const successCount = result.success_count;
            const failCount = result.fail_count;
            if (successCount > 0 || failCount > 0) {
                ElMessage.success(
                    t('toolbox.imageCompress.done', {
                        success: successCount,
                        fail: failCount,
                    }),
                );
            }
        } catch (error: any) {
            ElMessage.error(typeof error === 'string' ? error : String(error));
        } finally {
            if (unlisten) unlisten();
            loading.value = false;
        }
    }

    const totalRatio = computed(() => {
        if (totalOriginal.value > 0) {
            return ((totalOriginal.value - totalCompressed.value) / totalOriginal.value) * 100;
        }
        return 0;
    });

    const doneCount = computed(() => images.value.filter((i) => i.status === 'done').length);

    return {
        images,
        quality,
        format,
        scale,
        loading,
        totalOriginal,
        totalCompressed,
        totalRatio,
        doneCount,
        scanDir,
        scanFiles,
        removeImage,
        clearImages,
        compress,
    };
}
