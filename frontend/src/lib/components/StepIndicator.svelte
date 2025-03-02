<script lang="ts">
	// Step status types
	export let status: 'pending' | 'loading' | 'completed' | 'error' = 'pending';
	export let text: string;
	export let showSpinner = false;
</script>

<div class="mb-3 flex items-center">
	<!-- Status indicator -->
	{#if status === 'loading'}
		<div class="mr-3 h-6 w-6 flex-shrink-0">
			<div
				class="h-6 w-6 animate-spin rounded-full border-4 border-gray-300 border-t-blue-500"
			></div>
		</div>
	{:else if status === 'completed'}
		<div class="mr-3 flex h-6 w-6 flex-shrink-0 items-center justify-center text-green-500">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
			</svg>
		</div>
	{:else if status === 'error'}
		<div class="mr-3 flex h-6 w-6 flex-shrink-0 items-center justify-center text-red-500">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-6 w-6"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M6 18L18 6M6 6l12 12"
				/>
			</svg>
		</div>
	{:else}
		<div
			class="mr-3 h-6 w-6 flex-shrink-0 rounded-full border-2 border-gray-300 text-gray-300"
		></div>
	{/if}

	<!-- Step text -->
	<span
		class={status === 'completed'
			? 'text-gray-600'
			: status === 'error'
				? 'text-red-600'
				: 'text-gray-800'}
	>
		{text}
	</span>

	<!-- Optional spinner (for current step) -->
	{#if showSpinner && status === 'loading'}
		<div class="ml-2">
			<div class="inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500"></div>
			<div
				class="ml-1 inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500"
				style="animation-delay: 0.2s;"
			></div>
			<div
				class="ml-1 inline-block h-2 w-2 animate-pulse rounded-full bg-blue-500"
				style="animation-delay: 0.4s;"
			></div>
		</div>
	{/if}
</div>
