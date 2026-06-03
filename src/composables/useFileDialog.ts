import { open, save } from '@tauri-apps/plugin-dialog';

/**
 * 文件/文件夹选择器 composable
 *
 * 注意：该 composable 依赖 @tauri-apps/plugin-dialog，需在 Tauri 环境中运行。
 * npm 依赖已加入 package.json，首次使用请执行 `npm install`。
 */

export function useFileDialog() {
    /** 选择单个文件，返回文件路径字符串；用户取消返回 null */
    async function selectFile(options?: {
        title?: string;
        extensions?: string[];
        defaultPath?: string;
    }): Promise<string | null> {
        try {
            const selected = await open({
                title: options?.title ?? '选择文件',
                multiple: false,
                directory: false,
                defaultPath: options?.defaultPath,
                filters: options?.extensions
                    ? [{ name: '文件', extensions: options.extensions }]
                    : undefined,
            });
            return selected as string | null;
        } catch {
            return null;
        }
    }

    /** 选择文件夹，返回文件夹路径字符串；用户取消返回 null */
    async function selectFolder(options?: {
        title?: string;
        defaultPath?: string;
    }): Promise<string | null> {
        try {
            const selected = await open({
                title: options?.title ?? '选择文件夹',
                directory: true,
                multiple: false,
                defaultPath: options?.defaultPath,
            });
            return selected as string | null;
        } catch {
            return null;
        }
    }

    /** 保存文件对话框，返回保存路径字符串；用户取消返回 null */
    async function selectFileSave(options?: {
        title?: string;
        extensions?: string[];
        defaultPath?: string;
    }): Promise<string | null> {
        try {
            return await save({
                title: options?.title ?? '保存文件',
                defaultPath: options?.defaultPath,
                filters: options?.extensions
                    ? [{ name: '文件', extensions: options.extensions }]
                    : undefined,
            });
        } catch {
            return null;
        }
    }

    return { selectFile, selectFolder, selectFileSave };
}