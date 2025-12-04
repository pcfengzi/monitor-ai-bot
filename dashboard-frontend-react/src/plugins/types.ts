// src/plugins/types.ts

import React from 'react';

/**
 * Defines the contract for a frontend plugin.
 * Each plugin provides metadata for routing, navigation, and rendering.
 */
export interface FrontendPlugin {
  /**
   * A unique identifier for the plugin.
   * e.g., 'workflow-engine'
   */
  id: string;

  /**
   * The name to be displayed in the UI (e.g., navigation menu).
   * e.g., 'Workflow Management'
   */
  name: string;

  /**
   * The URL path for the plugin's main component.
   * e.g., '/workflows'
   */
  path: string;

  /**
   * The main React component to render for this plugin.
   */
  component: React.ComponentType;

  /**
   * An optional icon component for the navigation menu.
   */
  icon?: React.ReactNode;

  /**
   * Determines the order in the navigation menu. Lower numbers appear first.
   */
  order?: number;
}
