export type MenuBarRightItem = 'notification' | 'clipboard' | 'credits' | 'theme' | 'locale' | 'battery' | 'clock';

export interface MenuBarConfig {
    rightVisible: Record<MenuBarRightItem, boolean>;
}

export const menuBarConfig: MenuBarConfig = {
    rightVisible: {
        notification: true,
        clipboard: true,
        credits: true,
        theme: true,
        locale: true,
        battery: true,
        clock: true,
    },
};
