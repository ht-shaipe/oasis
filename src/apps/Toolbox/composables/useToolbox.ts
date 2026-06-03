import { ref } from 'vue';
import { TOOLS } from '../constants';

export function useToolbox() {
    const activeTool = ref<string>(TOOLS[0]?.id || '');

    function setActiveTool(toolId: string) {
        activeTool.value = toolId;
    }

    return {
        activeTool,
        setActiveTool,
        tools: TOOLS,
    };
}
