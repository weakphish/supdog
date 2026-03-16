

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.BqaEgYnK.js","_app/immutable/chunks/Bc0mxiO4.js","_app/immutable/chunks/D5QHtcP-.js","_app/immutable/chunks/OX6Gv7Kw.js"];
export const stylesheets = [];
export const fonts = [];
