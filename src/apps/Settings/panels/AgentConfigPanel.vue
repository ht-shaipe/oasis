<template>
    <div class="section-panel">
        <el-tabs v-model="activeTab" class="agent-tabs">
            <el-tab-pane :label="t('agent.tabConfig')" name="config">
                <div class="config-toolbar">
                    <el-button type="primary" size="small" :loading="saving" @click="saveConfig">
                        {{ t('app.save') }}
                    </el-button>
                </div>

                <el-collapse v-model="activeSections" class="agent-collapse">
                    <el-collapse-item :title="t('agent.sectionModel')" name="model">
                        <div class="form-row">
                            <label class="form-label">{{ t('agent.model') }}</label>
                            <el-select
                                v-model="config.model"
                                filterable
                                allow-create
                                :placeholder="t('agent.modelPlaceholder')"
                                size="small"
                                class="form-control"
                            >
                                <el-option value="claude-sonnet-4-20250514" label="Claude Sonnet 4" />
                                <el-option value="claude-3-5-sonnet-20241022" label="Claude 3.5 Sonnet" />
                                <el-option value="claude-3-5-haiku-20241022" label="Claude 3.5 Haiku" />
                                <el-option value="claude-3-opus-20240229" label="Claude 3 Opus" />
                            </el-select>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.apiProvider') }}</label>
                            <el-select
                                v-model="config.apiProvider"
                                :placeholder="t('agent.apiProviderPlaceholder')"
                                size="small"
                                clearable
                                class="form-control"
                            >
                                <el-option value="anthropic" label="Anthropic" />
                                <el-option value="bedrock" label="Amazon Bedrock" />
                                <el-option value="vertex" label="Google Vertex AI" />
                            </el-select>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.smallModel') }}</label>
                            <el-input
                                v-model="config.smallModel"
                                :placeholder="t('agent.smallModelPlaceholder')"
                                size="small"
                                class="form-control"
                            />
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.largeModel') }}</label>
                            <el-input
                                v-model="config.largeModel"
                                :placeholder="t('agent.largeModelPlaceholder')"
                                size="small"
                                class="form-control"
                            />
                        </div>
                    </el-collapse-item>

                    <el-collapse-item :title="t('agent.sectionPermissions')" name="permissions">
                        <div class="form-row">
                            <label class="form-label">{{ t('agent.defaultMode') }}</label>
                            <el-select
                                v-model="config.permissions!.defaultMode"
                                :placeholder="t('agent.defaultModePlaceholder')"
                                size="small"
                                clearable
                                class="form-control"
                            >
                                <el-option value="default" label="Default" />
                                <el-option value="bypassPermissions" label="Bypass Permissions" />
                                <el-option value="plan" label="Plan" />
                            </el-select>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.allowPatterns') }}</label>
                            <div class="pattern-list">
                                <div v-for="(_pat, idx) in allowPatterns" :key="idx" class="pattern-item">
                                    <el-input v-model="allowPatterns[idx]" size="small" class="pattern-input" />
                                    <el-button type="danger" link size="small" @click="removePattern(allowPatterns, idx)">
                                        <el-icon><Delete /></el-icon>
                                    </el-button>
                                </div>
                                <el-button size="small" plain @click="addPattern(allowPatterns)">
                                    {{ t('agent.addPattern') }}
                                </el-button>
                            </div>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.denyPatterns') }}</label>
                            <div class="pattern-list">
                                <div v-for="(_pat, idx) in denyPatterns" :key="idx" class="pattern-item">
                                    <el-input v-model="denyPatterns[idx]" size="small" class="pattern-input" />
                                    <el-button type="danger" link size="small" @click="removePattern(denyPatterns, idx)">
                                        <el-icon><Delete /></el-icon>
                                    </el-button>
                                </div>
                                <el-button size="small" plain @click="addPattern(denyPatterns)">
                                    {{ t('agent.addPattern') }}
                                </el-button>
                            </div>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.additionalDirs') }}</label>
                            <div class="pattern-list">
                                <div v-for="(_dir, idx) in additionalDirs" :key="idx" class="pattern-item">
                                    <el-input v-model="additionalDirs[idx]" size="small" class="pattern-input" />
                                    <el-button type="danger" link size="small" @click="removePattern(additionalDirs, idx)">
                                        <el-icon><Delete /></el-icon>
                                    </el-button>
                                </div>
                                <el-button size="small" plain @click="addPattern(additionalDirs)">
                                    {{ t('agent.addDir') }}
                                </el-button>
                            </div>
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.sandbox') }}</label>
                            <el-switch v-model="sandboxEnabled" />
                        </div>

                        <template v-if="sandboxEnabled">
                            <div class="form-row">
                                <label class="form-label">{{ t('agent.allowCommand') }}</label>
                                <div class="pattern-list">
                                    <div v-for="(_cmd, idx) in allowCommands" :key="idx" class="pattern-item">
                                        <el-input v-model="allowCommands[idx]" size="small" class="pattern-input" />
                                        <el-button type="danger" link size="small" @click="removePattern(allowCommands, idx)">
                                            <el-icon><Delete /></el-icon>
                                        </el-button>
                                    </div>
                                    <el-button size="small" plain @click="addPattern(allowCommands)">
                                        {{ t('agent.addCommand') }}
                                    </el-button>
                                </div>
                            </div>

                            <div class="form-row">
                                <label class="form-label">{{ t('agent.denyCommand') }}</label>
                                <div class="pattern-list">
                                    <div v-for="(_cmd, idx) in denyCommands" :key="idx" class="pattern-item">
                                        <el-input v-model="denyCommands[idx]" size="small" class="pattern-input" />
                                        <el-button type="danger" link size="small" @click="removePattern(denyCommands, idx)">
                                            <el-icon><Delete /></el-icon>
                                        </el-button>
                                    </div>
                                    <el-button size="small" plain @click="addPattern(denyCommands)">
                                        {{ t('agent.addCommand') }}
                                    </el-button>
                                </div>
                            </div>

                            <div class="form-row">
                                <label class="form-label">{{ t('agent.sandboxNetwork') }}</label>
                                <el-switch v-model="sandboxNetwork" />
                            </div>
                        </template>
                    </el-collapse-item>

                    <el-collapse-item :title="t('agent.sectionEnv')" name="env">
                        <div class="env-list">
                            <div v-for="(entry, idx) in envEntries" :key="idx" class="env-item">
                                <el-input
                                    v-model="entry.key"
                                    :placeholder="t('agent.envKey')"
                                    size="small"
                                    class="env-key-input"
                                />
                                <el-input
                                    v-model="entry.value"
                                    :placeholder="t('agent.envValue')"
                                    size="small"
                                    class="env-value-input"
                                />
                                <el-button type="danger" link size="small" @click="envEntries.splice(idx, 1)">
                                    <el-icon><Delete /></el-icon>
                                </el-button>
                            </div>
                            <el-button size="small" plain @click="envEntries.push({ key: '', value: '' })">
                                {{ t('agent.addEnv') }}
                            </el-button>
                        </div>
                    </el-collapse-item>

                    <el-collapse-item :title="t('agent.sectionMcp')" name="mcp">
                        <el-alert
                            v-if="mcpJsonError"
                            :title="mcpJsonError"
                            type="error"
                            show-icon
                            :closable="false"
                            class="mb-3"
                        />
                        <el-input
                            v-model="mcpJsonText"
                            type="textarea"
                            :rows="10"
                            :placeholder="mcpPlaceholder"
                            size="small"
                            class="mcp-textarea"
                            @input="validateMcpJson"
                        />
                    </el-collapse-item>

                    <el-collapse-item :title="t('agent.sectionAdvanced')" name="advanced">
                        <div class="form-row">
                            <label class="form-label">{{ t('agent.verbose') }}</label>
                            <el-switch v-model="config.verbose" />
                        </div>

                        <div class="form-row">
                            <label class="form-label">{{ t('agent.maxTurns') }}</label>
                            <el-input-number
                                v-model="config.maxTurns"
                                :min="1"
                                :max="200"
                                size="small"
                                class="form-control"
                            />
                        </div>
                    </el-collapse-item>
                </el-collapse>
            </el-tab-pane>

            <el-tab-pane :label="t('agent.tabTemplates')" name="templates">
                <div class="template-section">
                    <h3 class="sub-heading">{{ t('agent.systemTemplates') }}</h3>
                    <div v-if="templatesLoading" class="template-loading">{{ t('agent.loading') }}</div>
                    <div v-else-if="systemTemplates.length === 0" class="template-empty">
                        {{ t('agent.noTemplates') }}
                    </div>
                    <div v-else class="template-grid">
                        <div
                            v-for="tpl in systemTemplates"
                            :key="tpl.name"
                            class="template-card"
                            @click="applyTemplate(tpl)"
                        >
                            <div class="template-card-name">{{ tpl.name }}</div>
                            <div class="template-card-desc">{{ tpl.description }}</div>
                        </div>
                    </div>
                </div>

                <div class="template-section">
                    <div class="sub-heading-row">
                        <h3 class="sub-heading">{{ t('agent.userPresets') }}</h3>
                        <el-button size="small" type="primary" plain @click="saveAsPreset">
                            {{ t('agent.savePreset') }}
                        </el-button>
                    </div>
                    <div v-if="userPresets.length === 0" class="template-empty">
                        {{ t('agent.noPresets') }}
                    </div>
                    <div v-else class="template-grid">
                        <div
                            v-for="preset in userPresets"
                            :key="preset.id"
                            class="template-card"
                        >
                            <div class="template-card-header">
                                <div class="template-card-name">{{ preset.name }}</div>
                                <div class="template-card-actions">
                                    <el-button type="primary" link size="small" @click="applyPreset(preset.id)">
                                        {{ t('agent.apply') }}
                                    </el-button>
                                    <el-button type="danger" link size="small" @click="deletePreset(preset.id)">
                                        <el-icon><Delete /></el-icon>
                                    </el-button>
                                </div>
                            </div>
                            <div class="template-card-desc">{{ preset.description }}</div>
                        </div>
                    </div>
                </div>
            </el-tab-pane>

            <el-tab-pane :label="t('agent.tabBackups')" name="backups">
                <div v-if="backupsLoading" class="template-loading">{{ t('agent.loading') }}</div>
                <div v-else-if="backups.length === 0" class="template-empty">
                    {{ t('agent.noBackups') }}
                </div>
                <div v-else class="backup-list">
                    <div v-for="backup in backups" :key="backup.path" class="backup-item">
                        <div class="backup-info">
                            <div class="backup-name">{{ backup.name }}</div>
                            <div class="backup-meta">{{ backup.date }}</div>
                        </div>
                        <el-button type="primary" size="small" plain @click="restoreBackup(backup.path)">
                            {{ t('agent.restore') }}
                        </el-button>
                    </div>
                </div>
            </el-tab-pane>
        </el-tabs>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Delete } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';

const { t } = useI18n();

interface ClaudePermissions {
    allow?: string[];
    deny?: string[];
    defaultMode?: string;
    additionalDirectories?: string[];
}

interface ClaudeSandbox {
    enabled?: boolean;
    allowCommand?: string[];
    denyCommand?: string[];
    network?: boolean;
}

interface ClaudeMcpServer {
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    cwd?: string;
    type?: string;
    url?: string;
}

interface ClaudeConfig {
    model?: string;
    env?: Record<string, string>;
    permissions?: ClaudePermissions;
    mcpServers?: Record<string, ClaudeMcpServer>;
    apiProvider?: string;
    smallModel?: string;
    largeModel?: string;
    sandbox?: ClaudeSandbox;
    verbose?: boolean;
    maxTurns?: number;
    hooks?: unknown;
}

interface ConfigTemplate {
    name: string;
    description: string;
    config: ClaudeConfig;
}

interface UserPreset {
    id: string;
    name: string;
    description: string;
}

interface BackupEntry {
    name: string;
    path: string;
    date: string;
}

const activeTab = ref('config');
const activeSections = ref<string[]>(['model', 'permissions', 'env', 'mcp', 'advanced']);
const saving = ref(false);

const config = reactive<ClaudeConfig>({
    model: '',
    apiProvider: '',
    smallModel: '',
    largeModel: '',
    verbose: false,
    maxTurns: undefined,
    permissions: {},
    sandbox: {},
    env: {},
    mcpServers: {},
});

const allowPatterns = ref<string[]>([]);
const denyPatterns = ref<string[]>([]);
const additionalDirs = ref<string[]>([]);
const sandboxEnabled = ref(false);
const allowCommands = ref<string[]>([]);
const denyCommands = ref<string[]>([]);
const sandboxNetwork = ref(false);

const envEntries = ref<{ key: string; value: string }[]>([]);

const mcpJsonText = ref('');
const mcpJsonError = ref('');
const mcpPlaceholder = `{
  "server-name": {
    "command": "npx",
    "args": ["-y", "@example/mcp-server"],
    "env": { "API_KEY": "..." }
  }
}`;

const templatesLoading = ref(false);
const systemTemplates = ref<ConfigTemplate[]>([]);
const userPresets = ref<UserPreset[]>([]);

const backupsLoading = ref(false);
const backups = ref<BackupEntry[]>([]);

const addPattern = (list: string[]) => {
    list.push('');
};

const removePattern = (list: string[], idx: number) => {
    list.splice(idx, 1);
};

const validateMcpJson = () => {
    if (!mcpJsonText.value.trim()) {
        mcpJsonError.value = '';
        return;
    }
    try {
        const parsed = JSON.parse(mcpJsonText.value);
        if (typeof parsed !== 'object' || Array.isArray(parsed)) {
            mcpJsonError.value = t('agent.mcpMustBeObject');
            return;
        }
        mcpJsonError.value = '';
    } catch {
        mcpJsonError.value = t('agent.mcpInvalidJson');
    }
};

const syncFormToConfig = () => {
    config.permissions = {
        ...(config.permissions || {}),
        allow: allowPatterns.value.filter(p => p.trim()),
        deny: denyPatterns.value.filter(p => p.trim()),
        additionalDirectories: additionalDirs.value.filter(d => d.trim()),
    };

    config.sandbox = {
        enabled: sandboxEnabled.value,
        allowCommand: allowCommands.value.filter(c => c.trim()),
        denyCommand: denyCommands.value.filter(c => c.trim()),
        network: sandboxNetwork.value,
    };

    const env: Record<string, string> = {};
    for (const entry of envEntries.value) {
        if (entry.key.trim()) {
            env[entry.key.trim()] = entry.value;
        }
    }
    config.env = env;

    if (mcpJsonText.value.trim() && !mcpJsonError.value) {
        try {
            config.mcpServers = JSON.parse(mcpJsonText.value);
        } catch {
            // skip
        }
    } else if (!mcpJsonText.value.trim()) {
        config.mcpServers = {};
    }
};

const syncConfigToForm = () => {
    const perms = config.permissions || {};
    allowPatterns.value = perms.allow ? [...perms.allow] : [];
    denyPatterns.value = perms.deny ? [...perms.deny] : [];
    additionalDirs.value = perms.additionalDirectories ? [...perms.additionalDirectories] : [];

    const sandbox = config.sandbox || {};
    sandboxEnabled.value = sandbox.enabled ?? false;
    allowCommands.value = sandbox.allowCommand ? [...sandbox.allowCommand] : [];
    denyCommands.value = sandbox.denyCommand ? [...sandbox.denyCommand] : [];
    sandboxNetwork.value = sandbox.network ?? false;

    const env = config.env || {};
    envEntries.value = Object.entries(env).map(([key, value]) => ({ key, value }));

    if (config.mcpServers && Object.keys(config.mcpServers).length > 0) {
        mcpJsonText.value = JSON.stringify(config.mcpServers, null, 2);
    } else {
        mcpJsonText.value = '';
    }
    mcpJsonError.value = '';
};

const loadConfig = async () => {
    try {
        const loaded = await invoke<ClaudeConfig>('agent_load_config');
        Object.assign(config, loaded || {});
        if (!config.permissions) config.permissions = {};
        if (!config.sandbox) config.sandbox = {};
        if (!config.env) config.env = {};
        if (!config.mcpServers) config.mcpServers = {};
        syncConfigToForm();
    } catch (e) {
        console.error('Failed to load agent config:', e);
    }
};

const saveConfig = async () => {
    syncFormToConfig();
    saving.value = true;
    try {
        await invoke('agent_save_config', { config });
        ElMessage.success(t('agent.saveSuccess'));
    } catch (e: any) {
        ElMessage.error(e?.toString() || t('agent.saveFailed'));
    } finally {
        saving.value = false;
    }
};

const loadTemplates = async () => {
    templatesLoading.value = true;
    try {
        const [templates, presets] = await Promise.all([
            invoke<ConfigTemplate[]>('agent_list_config_templates'),
            invoke<UserPreset[]>('agent_list_presets'),
        ]);
        systemTemplates.value = templates || [];
        userPresets.value = presets || [];
    } catch (e) {
        console.error('Failed to load templates:', e);
    } finally {
        templatesLoading.value = false;
    }
};

const applyTemplate = async (tpl: ConfigTemplate) => {
    try {
        await invoke('agent_save_config', { config: tpl.config });
        Object.assign(config, tpl.config);
        if (!config.permissions) config.permissions = {};
        if (!config.sandbox) config.sandbox = {};
        if (!config.env) config.env = {};
        if (!config.mcpServers) config.mcpServers = {};
        syncConfigToForm();
        ElMessage.success(t('agent.templateApplied'));
    } catch (e: any) {
        ElMessage.error(e?.toString() || t('agent.applyFailed'));
    }
};

const saveAsPreset = async () => {
    try {
        const { value: name } = await ElMessageBox.prompt(
            t('agent.presetNameHint'),
            t('agent.savePreset'),
            { confirmButtonText: t('app.save'), cancelButtonText: t('app.cancel') },
        );
        if (!name?.trim()) return;
        syncFormToConfig();
        await invoke('agent_save_preset', { name: name.trim(), config });
        await loadTemplates();
        ElMessage.success(t('agent.presetSaved'));
    } catch {
        // cancelled
    }
};

const applyPreset = async (id: string) => {
    try {
        const loaded = await invoke<ClaudeConfig>('agent_apply_preset', { id });
        Object.assign(config, loaded || {});
        if (!config.permissions) config.permissions = {};
        if (!config.sandbox) config.sandbox = {};
        if (!config.env) config.env = {};
        if (!config.mcpServers) config.mcpServers = {};
        syncConfigToForm();
        ElMessage.success(t('agent.presetApplied'));
    } catch (e: any) {
        ElMessage.error(e?.toString() || t('agent.applyFailed'));
    }
};

const deletePreset = async (id: string) => {
    try {
        await ElMessageBox.confirm(
            t('agent.deletePresetConfirm'),
            t('agent.deletePreset'),
            { type: 'warning' },
        );
        await invoke('agent_delete_preset', { id });
        userPresets.value = userPresets.value.filter(p => p.id !== id);
        ElMessage.success(t('agent.presetDeleted'));
    } catch {
        // cancelled
    }
};

const loadBackups = async () => {
    backupsLoading.value = true;
    try {
        const list = await invoke<BackupEntry[]>('agent_list_backups');
        backups.value = list || [];
    } catch (e) {
        console.error('Failed to load backups:', e);
    } finally {
        backupsLoading.value = false;
    }
};

const restoreBackup = async (backupPath: string) => {
    try {
        await ElMessageBox.confirm(
            t('agent.restoreConfirm'),
            t('agent.restore'),
            { type: 'warning' },
        );
        const loaded = await invoke<ClaudeConfig>('agent_restore_backup', { backupPath });
        Object.assign(config, loaded || {});
        if (!config.permissions) config.permissions = {};
        if (!config.sandbox) config.sandbox = {};
        if (!config.env) config.env = {};
        if (!config.mcpServers) config.mcpServers = {};
        syncConfigToForm();
        ElMessage.success(t('agent.restoreSuccess'));
    } catch (e: any) {
        if (e !== 'cancel') {
            ElMessage.error(e?.toString() || t('agent.restoreFailed'));
        }
    }
};

onMounted(() => {
    loadConfig();
    loadTemplates();
    loadBackups();
});
</script>

<style scoped>
.section-panel {
    animation: sectionFadeIn 0.15s ease;
    padding: 0 12px 12px;
}

@keyframes sectionFadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
}

.section-heading {
    font-size: var(--app-font-16);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
    padding: 0 12px;
}

.agent-tabs :deep(.el-tabs__header) {
    margin-bottom: 16px;
}

.agent-tabs :deep(.el-tabs__item) {
    font-size: var(--app-font-13);
    font-weight: 500;
}

.config-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 12px;
}

.agent-collapse {
    border: none;
}

.agent-collapse :deep(.el-collapse-item__header) {
    font-size: var(--app-font-13);
    font-weight: 600;
    color: var(--color-text-primary);
    background: transparent;
    border-bottom: 1px solid var(--color-card-border);
    height: 36px;
    line-height: 36px;
}

.agent-collapse :deep(.el-collapse-item__wrap) {
    border-bottom: none;
    background: transparent;
}

.agent-collapse :deep(.el-collapse-item__content) {
    padding: 12px 0 4px;
}

.form-row {
    display: flex;
    align-items: flex-start;
    margin-bottom: 14px;
    gap: 12px;
}

.form-label {
    font-size: var(--app-font-13);
    font-weight: 500;
    color: var(--color-text-primary);
    min-width: 120px;
    padding-top: 5px;
    flex-shrink: 0;
}

.form-control {
    flex: 1;
    min-width: 0;
}

.pattern-list {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.pattern-item {
    display: flex;
    align-items: center;
    gap: 6px;
}

.pattern-input {
    flex: 1;
    min-width: 0;
}

.env-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.env-item {
    display: flex;
    align-items: center;
    gap: 8px;
}

.env-key-input {
    width: 140px;
    flex-shrink: 0;
}

.env-value-input {
    flex: 1;
    min-width: 0;
}

.mcp-textarea :deep(.el-textarea__inner) {
    font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
    font-size: var(--app-font-13);
    line-height: 1.6;
}

.template-section {
    margin-bottom: 24px;
}

.template-section:last-child {
    margin-bottom: 0;
}

.sub-heading {
    font-size: var(--app-font-13);
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
}

.sub-heading-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
}

.sub-heading-row .sub-heading {
    margin-bottom: 0;
}

.template-loading,
.template-empty {
    padding: 32px 0;
    text-align: center;
    font-size: var(--app-font-13);
    color: var(--color-text-tertiary);
}

.template-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
}

.template-card {
    padding: 14px 16px;
    border-radius: 10px;
    background: var(--color-card-bg);
    border: 1px solid var(--color-card-border);
    cursor: pointer;
    transition: box-shadow 0.15s;
}

.template-card:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.template-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.template-card-name {
    font-size: var(--app-font-13);
    font-weight: 600;
    color: var(--color-text-primary);
}

.template-card-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
}

.template-card-desc {
    font-size: var(--app-font-13);
    color: var(--color-text-tertiary);
    margin-top: 4px;
    line-height: 1.4;
}

.backup-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.backup-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-radius: 10px;
    background: var(--color-card-bg);
    border: 1px solid var(--color-card-border);
}

.backup-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.backup-name {
    font-size: var(--app-font-13);
    font-weight: 500;
    color: var(--color-text-primary);
}

.backup-meta {
    font-size: var(--app-font-13);
    color: var(--color-text-tertiary);
}
</style>
