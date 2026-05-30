<template>
    <MacWindow 
        :title="t('notes.helpTitle')" 
        :isMinimized="isMinimized" 
        @close="closeApp" 
        @minimize="toggleMinimize"
        width="800"
        height="600"
    >
        <div class="notes-container">
            <div class="notes-sidebar">
                <div class="sidebar-header">
                    <h3>{{ t('notes.title') }}</h3>
                    <button class="add-note-btn" :title="t('notes.newNote')">
                        <i class="el-icon-plus"></i>
                    </button>
                </div>
                <div class="notes-list">
                    <div class="note-item active">
                        <div class="note-title">{{ t('notes.helpTitle') }}</div>
                        <div class="note-preview">{{ t('notes.helpPreview') }}</div>
                    </div>
                </div>
            </div>
            <div class="notes-content">
                <div class="note-editor">
                    <h1>{{ t('notes.mainTitle') }}</h1>
                    
                    <h2>{{ t('notes.introduction') }}</h2>
                    <p>{{ t('notes.introDesc') }}</p>
                    
                    <h2>{{ t('help.basicUsage') }}</h2>
                    <ol>
                        <li><strong>{{ t('notes.step0') }}</strong></li>
                        <li><strong>{{ t('notes.openCodeGenerator') }}</strong></li>
                        <li><strong>{{ t('notes.describeNeeds') }}</strong>{{ t('notes.step2Desc') }}</li>
                        <li><strong>{{ t('notes.selectConfig') }}</strong></li>
                        <li><strong>{{ t('notes.step4Desc') }}</strong></li>
                        <li><strong>{{ t('notes.step5Desc') }}</strong></li>
                    </ol>
                    
                    <h2>{{ t('help.features') }}</h2>
                    <h3>{{ t('notes.codeGeneratorTitle') }}</h3>
                    <p>{{ t('notes.codeGeneratorDesc') }}</p>
                    <ul>
                        <li><strong>{{ t('notes.aiModel') }}</strong></li>
                        <li><strong>{{ t('notes.uiFramework') }}</strong></li>
                        <li><strong>{{ t('notes.jsFramework') }}</strong></li>
                        <li><strong>{{ t('notes.cdnSource') }}</strong></li>
                        <li><strong>{{ t('notes.deviceAdapt') }}</strong></li>
                    </ul>
                    
                    <h3>{{ t('notes.editorTitle') }}</h3>
                    <p>{{ t('notes.editorDesc') }}</p>
                    <ul>
                        <li>{{ t('codeEditor.syntaxHighlight') }}</li>
                        <li>{{ t('generator.codeFormatting') }}</li>
                        <li>{{ t('generator.autoSave') }}</li>
                    </ul>
                    
                    <h3>{{ t('notes.previewTitle') }}</h3>
                    <p>{{ t('notes.previewDesc') }}</p>
                    
                    <h3>{{ t('notes.versionTitle') }}</h3>
                    <p>{{ t('notes.versionDesc') }}</p>
                    
                    <h2>{{ t('help.tips') }}</h2>
                    <ul>
                        <li><strong>{{ t('notes.detailDesc') }}</strong></li>
                        <li><strong>{{ t('notes.specifyStack') }}</strong></li>
                        <li><strong>{{ t('notes.continueConv') }}</strong></li>
                        <li><strong>{{ t('notes.saveVersion') }}</strong></li>
                    </ul>
                    
                    <h2>{{ t('help.faq') }}</h2>
                    <p><strong>{{ t('notes.faq3FullQ') }}</strong></p>
                    <p>{{ t('notes.faq3FullA') }}</p>
                    
                    <p><strong>{{ t('notes.faq1FullQ') }}</strong></p>
                    <p>{{ t('notes.faq1FullA') }}</p>
                    
                    <p><strong>{{ t('notes.faq2FullQ') }}</strong></p>
                    <p>{{ t('notes.faq2FullA') }}</p>
                </div>
            </div>
        </div>
    </MacWindow>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import MacWindow from '@/components/common/MacWindow.vue';

const { t } = useI18n();

// 定义属性
const props = defineProps({
    isMinimized: {
        type: Boolean,
        default: false
    }
});

// 事件发射
const emit = defineEmits(['close', 'minimize']);

// 关闭应用
const closeApp = () => {
    emit('close');
};

// 切换最小化状态
const toggleMinimize = () => {
    emit('minimize');
};
</script>

<style scoped>
.notes-container {
    display: flex;
    height: 100%;
    background-color: #f5f5f5;
    border-radius: 0 0 10px 10px;
}

.notes-sidebar {
    width: 210px;
    background-color: var(--color-sidebar-bg);
    border-right: 1px solid var(--color-window-titlebar-border);
    display: flex;
    flex-direction: column;
}

.sidebar-header {
    padding: 15px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--color-window-titlebar-border);
}

.sidebar-header h3 {
    margin: 0;
    color: var(--color-text-primary);
}

.add-note-btn {
    background: none;
    border: none;
    font-size: 18px;
    color: #007aff;
    cursor: pointer;
}

.notes-list {
    flex: 1;
    overflow-y: auto;
}

.note-item {
    padding: 15px;
    border-bottom: 1px solid var(--color-window-titlebar-border);
    cursor: pointer;
}

.note-item:hover {
    background-color: var(--color-input-bg);
}

.note-item.active {
    background-color: #e2e2e2;
}

.note-title {
    font-weight: bold;
    color: var(--color-text-primary);
    margin-bottom: 5px;
}

.note-preview {
    color: var(--color-text-secondary);
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.notes-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.note-editor {
    flex: 1;
    padding: 20px;
    background: white;
    overflow-y: auto;
    line-height: 1.6;
}

.note-editor h1 {
    color: var(--color-text-primary);
    font-size: 24px;
    margin-top: 0;
    margin-bottom: 20px;
}

.note-editor h2 {
    color: #444;
    font-size: 20px;
    margin-top: 25px;
    margin-bottom: 15px;
    border-bottom: 1px solid #eee;
    padding-bottom: 5px;
}

.note-editor h3 {
    color: #555;
    font-size: 16px;
    margin-top: 20px;
    margin-bottom: 10px;
}

.note-editor p {
    margin-bottom: 15px;
}

.note-editor ul, .note-editor ol {
    margin-bottom: 15px;
    padding-left: 25px;
}

.note-editor li {
    margin-bottom: 8px;
}

.note-editor strong {
    font-weight: 600;
}
</style>
