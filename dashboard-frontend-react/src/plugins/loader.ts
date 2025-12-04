// src/plugins/loader.ts

// This function will dynamically import all plugin entry points.
// For now, we will hardcode the imports. In a more advanced system,
// this could be based on a configuration file or directory scan.
export const loadPlugins = () => {
  // Import the workflow plugin to trigger its registration.
  import('./workflow');
  
  // To add a new plugin, you would simply add a new import here:
  // import('./my-new-plugin');
};
