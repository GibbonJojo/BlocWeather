<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>Subregions - BlocWeather</title>
</svelte:head>

<div class="space-y-6">
	<!-- Back button -->
	<a href="/" class="inline-flex items-center text-blue-600 hover:text-blue-700 transition-colors">
		<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back to Countries
	</a>

	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold text-gray-900">Subregions</h1>
		<p class="text-gray-600 mt-2">{data.subregions.length} {data.subregions.length === 1 ? 'region' : 'regions'} available</p>
	</div>

	<!-- Subregions grid -->
	{#if data.subregions.length === 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
			<span class="text-4xl block mb-2">📍</span>
			<p class="text-yellow-800 font-medium">No subregions found</p>
			<p class="text-yellow-600 text-sm mt-2">Spots might be available at the country level</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each data.subregions as subregion (subregion.id)}
				<a
					href="/subregions/{subregion.id}"
					class="block bg-white rounded-lg shadow-md hover:shadow-xl transition-shadow duration-300 overflow-hidden group"
				>
					<div class="p-6">
						<h3 class="text-xl font-bold text-gray-900 group-hover:text-blue-600 transition-colors mb-3">
							{subregion.name}
						</h3>
						<div class="flex items-center text-gray-600">
							<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
							</svg>
							<span class="text-sm">
								{subregion.spot_count} {subregion.spot_count === 1 ? 'spot' : 'spots'}
							</span>
						</div>
					</div>
					<div class="h-1 bg-gradient-to-r from-blue-500 to-green-500 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300"></div>
				</a>
			{/each}
		</div>
	{/if}
</div>
