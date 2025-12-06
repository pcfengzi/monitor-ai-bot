// src/plugins/loader.ts

// 自动加载 src/plugins/*/entry.ts
const modules = import.meta.glob("./*/entry.ts", {
  eager: true,
});

export {};
