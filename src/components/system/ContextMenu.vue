<template>
    <div 
        class="context-menu" 
        v-show="visible" 
        :style="{ top: position.y + 'px', left: position.x + 'px' }"
        @click.stop
    >
        <div class="menu-group">
            <div class="menu-item" @click="handleViewAction">
                <span>{{ t('app.view') }}</span>
            </div>
            <div class="menu-item" @click="handleSortAction">
                <span>{{ t('finder.sort') }}</span>
            </div>
            <div class="menu-item" @click="handleRefreshAction">
                <span>{{ t('app.refresh') }}</span>
            </div>
        </div>
        <div class="menu-divider"></div>
        <div class="menu-group">
            <div class="menu-item" @click="handleCreateFileAction">
                <span>{{ t('codeEditor.newFile') }}</span>
            </div>
            <div class="menu-item" @click="handleCreateFolderAction">
                <span>{{ t('codeEditor.newFolder') }}</span>
            </div>
        </div>
        <div class="menu-divider"></div>
        <div class="menu-group">
            <div class="menu-item" @click="handleChangeWallpaper">
                <span>{{ t('contextMenu.changeWallpaper') }}</span>
            </div>
            <div class="menu-item" @click="handleDisplaySettings">
                <span>{{ t('contextMenu.displaySettings') }}</span>
            </div>
        </div>
        <div class="menu-divider"></div>
        <div class="menu-group">
            <div class="menu-item" @click="handlePersonalizeAction">
                <span>个性化</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { nextTick } from 'vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps({
    visible: {
        type: Boolean,
        default: false
    },
    position: {
        type: Object,
        default: () => ({ x: 0, y: 0 })
    }
});

const emit = defineEmits(['close']);

const handleViewAction = () => {
    emit('close');
};

const handleSortAction = () => {
    emit('close');
};

const handleRefreshAction = () => {
    window.location.reload();
    emit('close');
};

const handleCreateFileAction = () => {
    emit('close');
};

const handleCreateFolderAction = () => {
    emit('close');
};

// 随机切换壁纸
const handleChangeWallpaper = () => {
    const currentWallpaper = parseInt(localStorage.getItem('wallpaper') || '1');
    let newWallpaper;
    do {
        newWallpaper = Math.floor(Math.random() * 10) + 1;
    } while (newWallpaper === currentWallpaper);
    
    localStorage.setItem('wallpaper', newWallpaper.toString());
    
    const preloadImage = new Image();
    
    preloadImage.src = `/assets/wallpaper/${newWallpaper}.jpg`;
    
    preloadImage.onload = () => {
        nextTick(() => {
            const desktop = document.querySelector('.mac-desktop');
            if (desktop) {
                const el = desktop as HTMLElement;
                el.style.transition = 'background-image 1s ease-in-out';
                el.style.filter = 'blur(5px)';
                setTimeout(() => {
                    el.style.backgroundImage = `url('/assets/wallpaper/${newWallpaper}.jpg')`;
                    setTimeout(() => {
                        el.style.filter = 'none';
                    }, 300);
                    console.log(`壁纸已切换为: ${newWallpaper}.jpg`);
                }, 100);
            }
        });
    };
    
    preloadImage.onerror = () => {
        alert('壁纸加载失败，请检查图片路径是否正确');
    };
    
    emit('close');
};

const handleDisplaySettings = () => {
    emit('close');
};

const handlePersonalizeAction = () => {
    emit('close');
};

// 处理点击外部关闭菜单
const handleOutsideClick = (_event: MouseEvent) => {
    if (props.visible) {
        emit('close');
    }
};

// 阻止默认的浏览器右键菜单
const preventDefaultContextMenu = (event: MouseEvent) => {
    if (props.visible) {
        event.preventDefault();
    }
};

// 组件挂载和卸载时绑定/解绑全局事件监听器
onMounted(() => {
    document.addEventListener('click', handleOutsideClick);
    document.addEventListener('contextmenu', preventDefaultContextMenu);
});

onUnmounted(() => {
    document.removeEventListener('click', handleOutsideClick);
    document.removeEventListener('contextmenu', preventDefaultContextMenu);
});
</script>

<style scoped>
.context-menu {
    position: fixed;
    background-color: var(--color-context-bg);
    backdrop-filter: blur(10px);
    border-radius: 8px;
    box-shadow: 0 4px 20px var(--color-shadow);
    min-width: 200px;
    z-index: 1000;
    user-select: none;
    animation: fade-in 0.15s ease-out;
    transform-origin: top left;
}

.menu-group {
    padding: 5px 0;
}

.menu-item {
    padding: 8px 15px;
    cursor: pointer;
    font-size: 14px;
    display: flex;
    align-items: center;
    transition: all 0.1s;
}

.menu-item:hover {
    background-color: var(--color-context-hover);
}

.menu-divider {
    height: 1px;
    background-color: var(--color-context-divider);
    margin: 2px 0;
}

@keyframes fade-in {
    from {
        opacity: 0;
        transform: scale(0.95);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
</style> 