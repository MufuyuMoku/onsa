// The interface is a single page application inside the Tauri window: no
// server rendering, and no prerendering either, so the whole window is served
// from one fallback document (SPEC §1).
export const ssr = false;
export const prerender = false;
export const trailingSlash = 'always';
