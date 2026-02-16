import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

const host = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env?.TAURI_DEV_HOST;
const leanViteCacheDir = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
	?.LEAN_VITE_CACHE_DIR;

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	cacheDir: leanViteCacheDir || undefined,

	// Vite options tailored for Tauri development
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**']
		}
	}
});
