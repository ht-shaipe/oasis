<template>
    <div class="flex h-full w-full items-center justify-center bg-[linear-gradient(135deg,#f5f7fa_0%,#c3cfe2_100%)]">
        <div
            v-loading="loading"
            :element-loading-text="loadingText"
            class="w-[400px] rounded-[20px] border border-[rgba(255,255,255,0.3)] bg-[rgba(255,255,255,0.72)] p-10 text-center shadow-[0_10px_40px_rgba(0,0,0,0.08)] backdrop-blur-[20px]">
            <div class="mb-5 text-[#409eff] drop-shadow-[0_2px_4px_rgba(64,158,255,0.2)]">
                <slot name="icon" />
            </div>
            <h2 class="mb-7 text-[22px] font-600 tracking-[-0.5px] text-[var(--color-text-primary)]">
                {{ title }}
            </h2>
            <slot />
        </div>
    </div>
</template>

<script setup lang="ts">
defineProps<{
    title: string;
    loading: boolean;
    loadingText: string;
}>();
</script>
