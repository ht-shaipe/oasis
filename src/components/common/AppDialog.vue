<template>
    <el-dialog
        v-bind="$attrs"
        :model-value="modelValue"
        :close-on-click-modal="resolvedCloseOnBackdrop"
        :close-on-press-escape="resolvedCloseOnEsc"
        @update:model-value="handleUpdateModelValue">
        <slot />
        <template v-for="slotName in slotNames" :key="slotName" #[slotName]="slotProps">
            <slot :name="slotName" v-bind="slotProps" />
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useSlots, watch } from 'vue';

defineOptions({
    inheritAttrs: false,
});

const props = withDefaults(
    defineProps<{
        modelValue: boolean;
        closeOnBackdrop?: boolean;
        closeOnEsc?: boolean;
        disableContextMenu?: boolean;
    }>(),
    {
        closeOnBackdrop: false,
        closeOnEsc: true,
        disableContextMenu: true,
    },
);

const emit = defineEmits<{
    (e: 'update:modelValue', value: boolean): void;
}>();

const slots = useSlots();

const resolvedCloseOnBackdrop = computed(() => props.closeOnBackdrop);
const resolvedCloseOnEsc = computed(() => props.closeOnEsc);
const slotNames = computed(() => Object.keys(slots).filter((slotName) => slotName !== 'default'));

const handleUpdateModelValue = (value: boolean) => {
    emit('update:modelValue', value);
};

const handleContextMenu = (event: MouseEvent) => {
    if (!props.disableContextMenu || !props.modelValue) {
        return;
    }

    event.preventDefault();
};

const syncContextMenuGuard = () => {
    if (props.disableContextMenu && props.modelValue) {
        document.addEventListener('contextmenu', handleContextMenu, true);
        return;
    }

    document.removeEventListener('contextmenu', handleContextMenu, true);
};

watch(
    () => [props.modelValue, props.disableContextMenu],
    () => syncContextMenuGuard(),
    { immediate: true },
);

onMounted(() => {
    syncContextMenuGuard();
});

onBeforeUnmount(() => {
    document.removeEventListener('contextmenu', handleContextMenu, true);
});
</script>
