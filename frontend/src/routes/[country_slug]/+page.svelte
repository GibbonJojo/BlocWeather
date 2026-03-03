<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.country.name} - BlocWeather</title>
	<meta name="description" content="Climbing areas in {data.country.name}" />
</svelte:head>

<div class="space-y-6">
	<a href="/" class="inline-flex items-center text-blue-600 hover:text-blue-700 transition-colors">
		<svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</a>

	<div>
		<h1 class="text-3xl font-bold text-gray-900">{data.country.name}</h1>
		<p class="text-gray-600 mt-1">{data.country.subregions.length} {data.country.subregions.length === 1 ? 'region' : 'regions'}</p>
	</div>

	{#if data.country.subregions.length === 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
			<p class="text-yellow-800 font-medium">No regions found</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each data.country.subregions as subregion (subregion.id)}
				<a
					href="/{data.country.slug}/{subregion.slug}"
					class="block bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200 overflow-hidden group"
				>
					<div class="p-5">
						<h3 class="text-lg font-semibold text-gray-900 group-hover:text-blue-600 transition-colors mb-1">
							{subregion.name}
						</h3>
						<p class="text-sm text-gray-500">{subregion.spot_count} {subregion.spot_count === 1 ? 'spot' : 'spots'}</p>
					</div>
					<div class="h-0.5 bg-gradient-to-r from-blue-500 to-green-500 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300"></div>
				</a>
			{/each}
		</div>
	{/if}
</div>
