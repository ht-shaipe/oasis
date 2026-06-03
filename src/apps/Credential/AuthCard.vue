<template>
    <div class="flex h-full w-full items-center justify-center bg-[linear-gradient(135deg,var(--color-auth-bg-start)_0%,var(--color-auth-bg-end)_100%)]">
        <div
            v-loading="loading"
            :element-loading-text="loadingText"
            class="w-[400px] rounded-[20px] border border-[var(--color-auth-card-border)] bg-[var(--color-auth-card-bg)] p-10 text-center shadow-[0_10px_40px_var(--color-auth-card-shadow)] backdrop-blur-[20px]">
            <div class="mb-5 text-[var(--color-link)] drop-shadow-[0_2px_4px_rgba(64,158,255,0.2)]">
                <slot name="icon" />
            </div>
            <h2 class="mb-7 text-[var(--app-font-22)] font-600 tracking-[-0.5px] text-[var(--color-text-primary)]">
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
