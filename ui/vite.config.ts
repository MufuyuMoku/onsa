import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// The interface is a static single page application loaded by the Tauri
// window; there is no server (SPEC section 1).
export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				// Runes everywhere except in dependencies. Can go in Svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({
				pages: 'build',
				assets: 'build',
				fallback: 'index.html',
				precompress: false,
				strict: true
			})
		})
	],
	// Cargo owns the console output while `tauri dev` runs.
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		watch: {
			ignored: ['**/src-tauri/**']
		}
	}
});
