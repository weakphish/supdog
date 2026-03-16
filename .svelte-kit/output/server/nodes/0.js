

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.CV9h06q9.js","_app/immutable/chunks/Bc0mxiO4.js","_app/immutable/chunks/D5QHtcP-.js","_app/immutable/chunks/DGfVVGiB.js"];
export const stylesheets = [];
export const fonts = [];
