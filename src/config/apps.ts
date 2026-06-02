import type { Component } from 'vue';
import Finder from '@/apps/Finder.vue';
import Generator from '@/apps/Generator.vue';
import CodeEditor from '@/apps/CodeEditor.vue';
import Preview from '@/apps/Safari.vue';
import Settings from '@/apps/Settings.vue';
import ContinueDialog from '@/apps/ContinueDialog.vue';
import Notes from '@/apps/Notes.vue';
import Profile from '@/apps/Profile.vue';
import Credential from '@/apps/Credential/index.vue';
import Toolbox from '@/apps/Toolbox/Index.vue';
import Browser from '@/apps/Browser.vue';

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
        id: 'settings',
        name: 'Settings',
        icon: '/assets/icons/Settings.svg',
        component: Settings,
        showInDock: true,
        showOnDesktop: true,
        nameKey: 'desktop.settings',
    },
    {
        id: 'credential-manager',
        name: 'Credential Manager',
        icon: '/assets/icons/Privacy.svg',
        component: Credential,
        showInDock: true,
        showOnDesktop: true,
        nameKey: 'dock.Credential',
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
    {
        id: 'toolbox',
        name: 'Toolbox',
        icon: '/assets/icons/Toolbox.svg',
        component: Toolbox,
        showInDock: true,
        showOnDesktop: true,
        nameKey: 'toolbox.title',
    },
    {
        id: 'browser',
        name: 'Browser Controller',
        icon: '/assets/icons/Chrome.svg',
        component: Browser,
        showInDock: true,
        nameKey: 'browser.title',
    },
];
