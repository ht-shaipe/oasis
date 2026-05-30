<template>
    <div class="mac-dock">
        <div class="dock-item" @click="openApp('Finder')">
            <img src="/assets/icons/Finder.svg" :alt="t('dock.finder')" :title="t('dock.finder')" />
        </div>
        <div class="dock-item" @click="openApp('generator')">
            <img src="/assets/icons/Reminders.svg" :alt="t('dock.codeGenerator')" :title="t('dock.codeGenerator')" />
        </div>
        <div class="dock-item" @click="openApp('editor')">
            <img src="/assets/icons/vscode.svg" :alt="t('dock.editor')" :title="t('dock.editor')" />
        </div>
        <div class="dock-item" @click="openApp('safari')">
            <img src="/assets/icons/Safari.svg" :alt="t('dock.safari')" :title="t('dock.safari')" />
        </div>
        <div class="dock-item" @click="openApp('about')">
            <img src="/assets/icons/Settings.svg" :alt="t('dock.about')" :title="t('dock.about')" />
        </div>
        <div class="dock-item" @click="openApp('credential-manager')">
            <img
                src="/assets/icons/Keychain.svg"
                :alt="t('dock.credentialManager')"
                :title="t('dock.credentialManager')" />
        </div>
        <!-- <div class="dock-item" @click="openApp('profile')">
            <img src="/assets/icons/Contacts.svg" :alt="t('dock.profile')" :title="t('dock.profile')" />
        </div> -->
        <!-- <div class="dock-item" @click="openSystemMail">
            <img src="/assets/icons/Mail.svg" :alt="t('dock.mail')" :title="t('dock.mail')" />
        </div>
        <div class="dock-item" @click="openWeb('https://photo.HongTui.cn/')">
            <img src="/assets/icons/Photos.svg" :alt="t('dock.photos')" :title="t('dock.photos')" />
        </div>
        <div class="dock-item" @click="openWeb('https://HongTui.cn/')">
            <img src="/assets/icons/Maps.svg" :alt="t('dock.maps')" :title="t('dock.maps')" />
        </div> -->
    </div>
</template>

<script setup lang="ts">
// 事件发射
const emit = defineEmits(['openApp']);

import { useI18n } from 'vue-i18n';

const { t } = useI18n();

// 打开App
const openApp = (app: string) => {
    emit('openApp', app);
};

// 打开网页
const openWeb = (url: string, target = '_self') => {
    emit('openApp', { type: 'safari', url: url, target: target });
};

// 打开系统邮件
const openSystemMail = () => {
    const email = 'HongTui@qq.com';
    const subject = encodeURIComponent('WebAI反馈Bug');
    const body = encodeURIComponent(
        '我在使用WebAI时发现了以下问题：\n\n[请在此描述您遇到的问题]\n\n系统信息：\n浏览器：' + navigator.userAgent,
    );
    window.location.href = `mailto:${email}?subject=${subject}&body=${body}`;
};
</script>

<style scoped>
/* Mac Dock栏 */
.mac-dock {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 15px;
    padding: 10px 20px;
    background-color: var(--color-dock-bg);
    border-radius: 16px;
    backdrop-filter: blur(10px);
    box-shadow: 0 8px 16px var(--color-dock-shadow);
    z-index: 1000;
}

.dock-item {
    width: 60px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
}

.dock-item img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}

.dock-item:hover {
    transform: translateY(-8px) scale(1.1);
}
</style>
