<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { initShinyListeners } from '$lib/utils/shinyListener';
	let { children } = $props();

	// Initialize Shiny listeners when the app starts
	onMount(() => {
		initShinyListeners();
		console.log('Shiny listeners initialized');
	});
</script>

{@render children()}
