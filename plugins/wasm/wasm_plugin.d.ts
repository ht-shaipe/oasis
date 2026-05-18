/* tslint:disable */
/* eslint-disable */

/**
 * 计数器插件
 */
export class CounterPlugin {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * 减少计数
     */
    decrement(): PluginState;
    /**
     * 获取状态
     */
    get_state(): PluginState;
    /**
     * 处理操作
     */
    handle_action(action: string, value: number): PluginState;
    /**
     * 增加计数
     */
    increment(): PluginState;
    /**
     * 获取插件元数据
     */
    manifest(): PluginManifest;
    constructor();
    /**
     * 重置
     */
    reset(): PluginState;
    /**
     * 设置最大值
     */
    set_max(max: number): PluginState;
}

/**
 * 插件元数据
 */
export class PluginManifest {
    free(): void;
    [Symbol.dispose](): void;
    constructor(id: string, title: string, icon: string, description: string, version: string);
    /**
     * 转换为 JSON
     */
    to_json(): string;
    readonly description: string;
    readonly icon: string;
    readonly id: string;
    readonly title: string;
    readonly version: string;
}

/**
 * 插件状态
 */
export class PluginState {
    free(): void;
    [Symbol.dispose](): void;
    constructor(data: string);
    readonly data: string;
}

/**
 * 创建插件实例
 */
export function create_plugin(): CounterPlugin;

/**
 * 获取所有可用插件
 */
export function get_available_plugins(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_counterplugin_free: (a: number, b: number) => void;
    readonly __wbg_pluginmanifest_free: (a: number, b: number) => void;
    readonly __wbg_pluginstate_free: (a: number, b: number) => void;
    readonly counterplugin_decrement: (a: number) => number;
    readonly counterplugin_get_state: (a: number) => number;
    readonly counterplugin_handle_action: (a: number, b: number, c: number, d: number) => number;
    readonly counterplugin_increment: (a: number) => number;
    readonly counterplugin_manifest: (a: number) => number;
    readonly counterplugin_new: () => number;
    readonly counterplugin_reset: (a: number) => number;
    readonly counterplugin_set_max: (a: number, b: number) => number;
    readonly get_available_plugins: (a: number) => void;
    readonly pluginmanifest_description: (a: number, b: number) => void;
    readonly pluginmanifest_icon: (a: number, b: number) => void;
    readonly pluginmanifest_id: (a: number, b: number) => void;
    readonly pluginmanifest_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => number;
    readonly pluginmanifest_title: (a: number, b: number) => void;
    readonly pluginmanifest_to_json: (a: number, b: number) => void;
    readonly pluginmanifest_version: (a: number, b: number) => void;
    readonly pluginstate_data: (a: number, b: number) => void;
    readonly pluginstate_new: (a: number, b: number) => number;
    readonly create_plugin: () => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
