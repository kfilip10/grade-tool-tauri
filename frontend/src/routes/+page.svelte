<script lang="ts">
	import { browser } from '$app/environment';
	import { Alert, Button, Spinner, Card, Progressbar } from 'flowbite-svelte';
	import UpdateProgress from '$lib/components/UpdateProgress.svelte';
	import { onMount, onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';

	import {
		updateProgressVisible,
		handleUpdateComplete,
		handleUpdateError
	} from '$lib/utils/updater';

	import { launchShinyApp, stopShinyApp } from '$lib/utils/shiny';
	import { initializeApp, initStatus, initMessage, initError } from '$lib/utils/initialization';
	import { shinyStatus, shinyUrl, shinyError, initShinyListeners } from '$lib/utils/shinyListener';

	let mode = 'updater';
	let passedShinyUrl: string | null = null;

	if (browser) {
		const urlParams = new URLSearchParams(window.location.search);
		mode = urlParams.get('mode') || 'updater';
		passedShinyUrl = urlParams.get('shinyUrl');

		console.log('Mode:', mode);
		console.log('Shiny URL from params:', passedShinyUrl);

		// If we're in app mode and have a passed Shiny URL, use it
		if (mode === 'app' && passedShinyUrl) {
			const decodedUrl = decodeURIComponent(passedShinyUrl);
			console.log('Setting Shiny URL to:', decodedUrl);
			shinyUrl.set(decodedUrl);
			shinyStatus.set('running');
		}
	}

	let rscriptPath = '';
	let autoLaunchShiny = true;
	let iframeEl: HTMLIFrameElement;

	async function fetchRscriptPath() {
		if (!browser) return; // Skip on server

		try {
			rscriptPath = await invoke<string>('get_rscript_path');
		} catch (error) {
			console.error('Failed to get Rscript path:', error);
			rscriptPath = 'Error retrieving Rscript path';
		}
	}

	// Start updater flow
	function startUpdater() {
		initializeApp();
	}

	// Set up closing behavior
	onMount(async () => {
		if (!browser) return; // Skip on server

		initShinyListeners();
		await fetchRscriptPath();

		const appWindow = getCurrentWindow();
		appWindow.onCloseRequested(async (event) => {
			if ($shinyStatus === 'running') {
				event.preventDefault(); // Prevent closing until Shiny is stopped
				try {
					await stopShinyApp();
					setTimeout(() => appWindow.close(), 500);
				} catch (e) {
					console.error('Error stopping Shiny on close:', e);
					appWindow.close(); // Close anyway if there's an error
				}
			}
		});

		// Auto-start updater flow if in updater mode
		if (mode === 'updater' && autoLaunchShiny) {
			startUpdater();
		}
	});

	import { stopBrowserMonitoring } from '$lib/utils/windowManager';

	// In onDestroy:
	onDestroy(async () => {
		if (!browser) return; // Skip on server

		// Make sure to stop monitoring
		stopBrowserMonitoring();

		// Make sure to stop Shiny when component is destroyed
		if ($shinyStatus === 'running') {
			try {
				await stopShinyApp();
			} catch (e) {
				console.error('Error stopping Shiny on destroy:', e);
			}
		}
	});
</script>

<!-- Render different UI based on mode -->
{#if mode === 'updater'}
	<!-- Updater Window UI -->
	<main class="p-4">
		<h1 class="mb-4 text-2xl font-bold">Grade Tool Setup</h1>

		<!-- Initialization Status -->
		{#if $initStatus === 'starting' || $initStatus === 'checking-updates'}
			<Card padding="xl" class="mb-4">
				<div class="flex items-center space-x-4">
					<Spinner size="6" />
					<div>
						<h3 class="font-semibold">{$initMessage}</h3>
						<p class="text-sm text-gray-500">Please wait while we get things ready...</p>
					</div>
				</div>
			</Card>
		{:else if $initStatus === 'downloading-update'}
			<Card padding="xl" class="mb-4">
				<h3 class="mb-2 font-semibold">{$initMessage}</h3>
				<Progressbar progress={25} size="h-2" class="mb-2" />
				<p class="text-sm text-gray-500">Please wait while the update downloads...</p>
			</Card>
		{:else if $initStatus === 'update-failed'}
			<Alert color="yellow" class="mb-4">
				<span class="font-medium">{$initMessage}</span>
			</Alert>
		{:else if $initStatus === 'launching-shiny'}
			<Card padding="xl" class="mb-4">
				<div class="flex items-center space-x-4">
					<Spinner size="6" />
					<div>
						<h3 class="font-semibold">Launching Shiny application...</h3>
						<p class="text-sm text-gray-500">Starting R and loading packages...</p>
					</div>
				</div>
			</Card>

			<!-- Add this section after the 'launching-shiny' state -->
		{:else if $initStatus === 'shiny-ready'}
			<Card padding="xl" class="mb-4">
				<div class="text-center">
					<div class="mb-2 flex items-center justify-center">
						<div class="mr-2 h-3 w-3 rounded-full bg-green-500"></div>
						<h3 class="font-semibold">Shiny application is running</h3>
					</div>
					<p class="mb-4 text-sm text-gray-500">The app is now open in your browser</p>

					<div class="mb-4 rounded-md border border-blue-200 bg-blue-50 p-3 text-left">
						<h4 class="mb-1 font-medium text-blue-800">Important:</h4>
						<p class="text-sm text-blue-600">
							This window monitors your Shiny session. Keep it open while using the app. When you
							close the browser tab, this window will automatically stop the Shiny process.
						</p>
					</div>

					<!-- Add buttons for common actions -->
					<div class="flex justify-center gap-2">
						<Button color="blue" on:click={() => $shinyUrl && open($shinyUrl)}>
							Reopen in Browser
						</Button>
						<Button color="red" on:click={stopShinyApp}>Stop Application</Button>
					</div>
				</div>
			</Card>
		{:else if $initStatus === 'error' || $shinyStatus === 'error'}
			<Alert color="red" class="mb-4">
				<span class="font-medium">Error: {$initError || $shinyError}</span>
				<div class="mt-2">
					<Button color="dark" size="xs" on:click={startUpdater}>Retry</Button>
				</div>
			</Alert>
		{/if}

		<!-- Update Progress Modal -->
		<UpdateProgress
			bind:open={$updateProgressVisible}
			onComplete={handleUpdateComplete}
			onError={handleUpdateError}
		/>
	</main>
{/if}
