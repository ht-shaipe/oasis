<template>
    <div class="section-panel">
        <h2 class="section-heading">{{ t('settings.general.title') }}</h2>

        <div class="setting-row">
            <div class="setting-info">
                <span class="setting-label">{{ t('settings.workspace.title') }}</span>
                <span class="setting-desc">{{ t('settings.workspace.desc') }}</span>
            </div>
        </div>
        <div class="setting-row">
            <div class="workspace-path-row">
                <input
                    class="workspace-input"
                    :value="workspaceDir"
                    readonly
                    :placeholder="t('settings.workspace.placeholder')"
                />
                <button class="workspace-browse-btn" @click="pickDirectory" :disabled="pickingDir">
                    {{ pickingDir ? '...' : t('settings.workspace.browse') }}
                </button>
            </div>
            <p v-if="workspaceStatus" class="workspace-status" :class="{ error: workspaceError }">
                {{ workspaceStatus }}
            </p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const { t } = useI18n();

const workspaceDir = ref('');
const pickingDir = ref(false);
const workspaceStatus = ref('');
const workspaceError = ref(false);

const loadWorkspaceDir = async () => {
    try {
        workspaceDir.value = await invoke<string>('get_workspace_dir');
        workspaceStatus.value = '';
        workspaceError.value = false;
    } catch (e) {
        workspaceStatus.value = String(e);
        workspaceError.value = true;
    }
};

const pickDirectory = async () => {
    pickingDir.value = true;
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: t('settings.workspace.selectTitle'),
        });
        if (selected) {
            const path = typeof selected === 'string' ? selected : selected;
            const result = await invoke<string>('set_workspace_dir', { path });
            workspaceDir.value = result;
            workspaceStatus.value = t('settings.workspace.saved');
            workspaceError.value = false;
            setTimeout(() => { workspaceStatus.value = ''; }, 3000);
        }
    } catch (e) {
        workspaceStatus.value = String(e);
        workspaceError.value = true;
    } finally {
        pickingDir.value = false;
    }
};

onMounted(() => {
    loadWorkspaceDir();
});
</script>
