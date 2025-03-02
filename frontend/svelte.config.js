import { mdsvex } from 'mdsvex';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: [vitePreprocess(), mdsvex()],

	kit: {
        adapter: adapter({
            fallback: 'index.html'
        }),
        // Required for Tauri static build
        prerender: {
            handleMissingId: 'ignore'
        }
	},

	extensions: ['.svelte', '.svx']
};

export default config;
