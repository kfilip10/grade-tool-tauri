<script lang="ts">
	import { Modal, Progressbar } from 'flowbite-svelte';
	import { onDestroy } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';

	export let open = false;
	export let onComplete = () => {};
	export let onError = (error: string) => {};

	let progress = 0;
	let downloaded = 0;
	let total = 0;
	let error = '';

	let interval: number;

	$: if (open) {
		startTracking();
	} else {
		stopTracking();
	}

	function startTracking() {
		interval = window.setInterval(async () => {
			try {
				const progressJson = await invoke<string>('get_update_progress');
				const progressData = JSON.parse(progressJson);

				progress = progressData.percent;
				downloaded = progressData.downloaded;
				total = progressData.total || 0;

				if (progressData.complete) {
					stopTracking();
					onComplete();
				} else if (progressData.error) {
					error = progressData.error;
					stopTracking();
					onError(error);
				}
			} catch (e) {
				console.error('Failed to get update progress', e);
			}
		}, 500);
	}

	function stopTracking() {
		if (interval) {
			clearInterval(interval);
		}
	}

	onDestroy(stopTracking);

	function formatBytes(bytes: number) {
		if (bytes === 0) return '0 B';
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(1024));
		return (bytes / Math.pow(1024, i)).toFixed(2) + ' ' + sizes[i];
	}
</script>

<Modal title="Downloading Update..." bind:open outsideclose={false} autoclose={false}>
	<div class="space-y-4">
		<Progressbar {progress} size="h-4" />

		<div class="text-center">
			<p class="text-lg font-semibold">{Math.round(progress)}%</p>
			{#if total > 0}
				<p class="text-sm text-gray-500">
					{formatBytes(downloaded)} of {formatBytes(total)}
				</p>
			{:else}
				<p class="text-sm text-gray-500">{formatBytes(downloaded)} downloaded</p>
			{/if}
		</div>

		{#if error}
			<div class="rounded bg-red-100 p-3 text-red-800">
				Error: {error}
			</div>
		{/if}
	</div>
</Modal>
