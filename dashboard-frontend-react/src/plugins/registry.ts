// src/plugins/registry.ts
import type { FrontendPlugin } from './types';

// A private array to hold all registered plugins.
const plugins: FrontendPlugin[] = [];

/**
 * Registers a new frontend plugin.
 * This should be called by each plugin's entry point.
 * @param plugin The plugin definition to register.
 */
export const registerPlugin = (plugin: FrontendPlugin) => {
  // Avoid duplicate registrations
  if (plugins.some(p => p.id === plugin.id)) {
    console.warn(`Plugin with id "${plugin.id}" is already registered.`);
    return;
  }
  plugins.push(plugin);
};

/**
 * Retrieves all registered plugins, sorted by the 'order' property.
 * @returns A sorted array of plugin definitions.
 */
export const getPlugins = (): FrontendPlugin[] => {
  return [...plugins].sort((a, b) => (a.order ?? 99) - (b.order ?? 99));
};
