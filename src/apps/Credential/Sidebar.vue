<template>
    <aside
        class="flex flex-col overflow-y-auto border-r border-[var(--color-window-titlebar-border)] bg-[var(--color-sidebar-bg)] py-2.5">
        <div class="flex-1">
            <div
                class="mb-2 flex items-center justify-between px-4 text-[var(--app-font-13)] font-600 uppercase tracking-[0.05em] text-[var(--color-text-tertiary)]">
                <span>{{ title }}</span>
                <el-button class="credential-sidebar-add" link @click="emit('add-category')">
                    <el-icon :size="18"><Plus /></el-icon>
                </el-button>
            </div>

            <div
                :class="['credential-sidebar-item', { active: selectedCategoryId === null }]"
                @click="emit('select-category', null)">
                <el-icon><FolderOpened /></el-icon>
                <span>{{ allLabel }}</span>
            </div>

            <el-tree
                :data="categoryTree"
                :props="{ label: 'name', children: 'children' }"
                node-key="id"
                default-expand-all
                :current-node-key="selectedCategoryId ?? undefined"
                highlight-current
                :indent="12"
                class="credential-category-tree"
                :expand-on-click-node="false"
                @node-click="handleNodeClick">
                <template #default="{ node, data }">
                    <div class="credential-tree-node group" :class="{ active: selectedCategoryId === data.id }">
                        <el-icon class="mr-1.5 text-[var(--app-font-16)] text-[#007aff]">
                            <component :is="node.expanded ? FolderOpened : Folder" />
                        </el-icon>
                        <span class="min-w-0 flex-1 truncate">{{ node.label }}</span>
                        <div class="ml-1 flex items-center opacity-0 transition-opacity group-hover:opacity-100">
                            <el-button
                                class="credential-tree-action"
                                link
                                size="small"
                                @click.stop="emit('quick-add-sub-category', data.id)">
                                <el-icon><Plus /></el-icon>
                            </el-button>
                            <el-button
                                class="credential-tree-action credential-tree-action-danger"
                                link
                                size="small"
                                @click.stop="emit('delete-category', data)">
                                <el-icon><Delete /></el-icon>
                            </el-button>
                        </div>
                    </div>
                </template>
            </el-tree>
        </div>
    </aside>
</template>

<script setup lang="ts">
import { Delete, Folder, FolderOpened, Plus } from '@element-plus/icons-vue';
import type { Category } from '@/composables/useCredential';

type CategoryTreeNode = Pick<Category, 'id' | 'name'> & { children: CategoryTreeNode[] };

defineProps<{
    title: string;
    allLabel: string;
    categoryTree: CategoryTreeNode[];
    selectedCategoryId: number | null;
}>();

const emit = defineEmits<{
    (e: 'add-category'): void;
    (e: 'select-category', categoryId: number | null): void;
    (e: 'quick-add-sub-category', parentId: number): void;
    (e: 'delete-category', category: Pick<Category, 'id' | 'name'>): void;
}>();

const handleNodeClick = (data: CategoryTreeNode) => {
    emit('select-category', data.id);
};
</script>

<style scoped>
.credential-sidebar-item {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    height: 30px;
    cursor: pointer;
    border-radius: 6px;
    margin: 0 8px 2px;
    transition: all 0.2s ease;
    font-size: var(--app-font-13);
    color: var(--color-text-primary);
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.credential-sidebar-item .el-icon {
    margin-right: 6px;
    font-size: var(--app-font-16);
    color: #007aff;
}

.credential-sidebar-item:hover {
    background-color: rgba(0, 0, 0, 0.05);
}

.credential-sidebar-item.active {
    background-color: #007aff;
    color: #ffffff;
}

.credential-sidebar-item.active .el-icon {
    color: #ffffff;
}

.credential-category-tree {
    background: transparent;
    padding: 0;
}

.credential-category-tree :deep(.el-tree-node__content) {
    height: 30px;
    padding: 0 !important;
    margin: 0 8px 2px;
    border-radius: 6px;
    transition: all 0.2s ease;
}

.credential-category-tree :deep(.el-tree-node__content:hover) {
    background-color: rgba(0, 0, 0, 0.05);
}

.credential-category-tree :deep(.el-tree-node.is-current > .el-tree-node__content) {
    background-color: transparent;
}

.credential-tree-node {
    display: flex;
    align-items: center;
    width: 100%;
    height: 100%;
    padding: 0 8px;
    border-radius: 6px;
    font-size: var(--app-font-13);
    position: relative;
}

.credential-tree-node.active {
    background-color: #007aff;
    color: #ffffff;
}

.credential-tree-node.active :deep(.el-icon),
.credential-tree-node.active span {
    color: #ffffff;
}

.credential-tree-action {
    padding: 0 2px;
    height: 20px;
    color: var(--color-text-secondary);
}

.credential-tree-action:hover {
    color: var(--el-color-primary);
}

.credential-tree-action-danger:hover {
    color: var(--el-color-danger);
}

.credential-sidebar-add {
    padding: 2px;
    height: 20px;
    width: 20px;
    color: var(--color-text-tertiary);
    border-radius: 4px;
    transition: all 0.2s;
}

.credential-sidebar-add:hover {
    background-color: rgba(0, 0, 0, 0.05);
    color: var(--el-color-primary);
}
</style>
