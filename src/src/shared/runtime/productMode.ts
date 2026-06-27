export const showDebugExperience = import.meta.env.DEV || import.meta.env.VITE_SHOW_DEBUG_EXPERIENCE === 'true'

export const showAiToolsEntry = showDebugExperience || import.meta.env.VITE_SHOW_AI_TOOLS === 'true'
