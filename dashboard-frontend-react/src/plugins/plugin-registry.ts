// src/plugins/plugin-registry.ts
import type { FrontendPlugin } from "./types";

const plugins: FrontendPlugin[] = [];

export const registerPlugin = (plugin: FrontendPlugin) => {
  if (plugins.some((p) => p.id === plugin.id)) {
    console.warn(`Plugin with id "${plugin.id}" is already registered.`);
    return;
  }
  plugins.push(plugin);
};

export const getPlugins = (): FrontendPlugin[] => {
  return [...plugins].sort((a, b) => (a.order ?? 99) - (b.order ?? 99));
};
