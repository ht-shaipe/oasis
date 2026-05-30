import type { Component } from 'vue';
import Finder from '@/apps/Finder.vue';
import Generator from '@/apps/Generator.vue';
import CodeEditor from '@/apps/CodeEditor.vue';
import Preview from '@/apps/Safari.vue';
import About from '@/apps/About.vue';
import ContinueDialog from '@/apps/ContinueDialog.vue';
import Notes from '@/apps/Notes.vue';
import Profile from '@/apps/Profile.vue';
import CredentialManager from '@/apps/CredentialManager/index.vue';

export interface AppConfig {
    id: string;
    name: string;
    icon: string;
    component: Component;
    showInDock?: boolean;
    showOnDesktop?: boolean;
    nameKey?: string; // 用于 i18n
}

export const apps: AppConfig[] = [
    {
        id: 'Finder',
        name: 'Finder',
        icon: '/assets/icons/Finder.svg',
        component: Finder,
        showInDock: true,
        nameKey: 'dock.finder',
    },
    {
        id: 'generator',
        name: 'Generator',
        icon: '/assets/icons/Reminders.svg',
        component: Generator,
        showInDock: true,
        nameKey: 'dock.codeGenerator',
    },
    {
        id: 'editor',
        name: 'Code Editor',
        icon: '/assets/icons/vscode.svg',
        component: CodeEditor,
        showInDock: true,
        nameKey: 'dock.editor',
    },
    {
        id: 'safari',
        name: 'Safari',
        icon: '/assets/icons/Safari.svg',
        component: Preview,
        showInDock: true,
        nameKey: 'dock.safari',
    },
    {
        id: 'about',
        name: 'About',
        icon: '/assets/icons/Settings.svg',
        component: About,
        showInDock: true,
        showOnDesktop: true,
        nameKey: 'desktop.about',
    },
    {
        id: 'credential-manager',
        name: 'Credential Manager',
        icon: '/assets/icons/Privacy.svg',
        component: CredentialManager,
        showInDock: true,
        showOnDesktop: true,
        nameKey: 'dock.credentialManager',
    },
    {
        id: 'notes',
        name: 'Notes',
        icon: '/assets/icons/Notes.svg',
        component: Notes,
        showOnDesktop: true,
        nameKey: 'contextMenu.useHelp',
    },
    {
        id: 'profile',
        name: 'Profile',
        icon: '/assets/icons/Contacts.svg',
        component: Profile,
        nameKey: 'dock.profile',
    },
    {
        id: 'continue-dialog',
        name: 'Continue Dialog',
        icon: '/assets/icons/Features.svg',
        component: ContinueDialog,
        nameKey: 'dock.features',
    },
];
