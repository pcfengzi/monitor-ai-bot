// src/plugins/workflow/index.ts
import { registerPlugin } from '../registry';
import { WorkflowPluginPage } from './WorkflowPluginPage';
import React from 'react';

// A simple placeholder icon
const WorkflowIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
    <path d="M10 4H4v6h6V4zm-2 4H6V6h2v2zm10 0h-6V4h6v6zm-2-4h-2v2h2V6zM8 20H4v-6h6v6H8zm-2-4H6v-2h2v2zm10 0h-6v-6h6v6zm-2-4h-2v2h2v-2z" fill="currentColor"/>
    <path d="M9 11.5H7.5v-1H11v4h1.5v1H11v1.5h-1V17h-1.5v-4H9v-1.5z" fill="currentColor"/>
  </svg>
);


registerPlugin({
  id: 'workflow-engine',
  name: '工作流管理',
  path: '/workflows',
  component: WorkflowPluginPage,
  icon: React.createElement(WorkflowIcon),
  order: 100,
});
