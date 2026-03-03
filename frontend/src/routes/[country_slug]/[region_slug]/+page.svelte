<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;

	$: country = data.subregion.country;
	$: spots = data.subregion.spots;
</script>

<svelte:head>
	<title>{data.subregion.name} - BlocWeather</title>
	<meta name="description" content="Climbing spots in {data.subregion.name}, {country.name}" />
</svelte:head>

<div class="space-y-6">
	<a
		href="/{country.slug}"
		class="inline-flex items-center text-blue-600 hover:text-blue-700 transition-colors"
	>
		<svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		{country.name}
	</a>

	<div>
		<h1 class="text-3xl font-bold text-gray-900">{data.subregion.name}</h1>
		<p class="text-gray-600 mt-1">{spots.length} {spots.length === 1 ? 'spot' : 'spots'}</p>
	</div>

	{#if spots.length === 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
			<p class="text-yellow-800 font-medium">No spots found</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			{#each spots as spot (spot.id)}
				<a
					href="/{country.slug}/{data.subregion.slug}/{spot.slug}"
					class="block bg-white rounded-lg shadow-sm hover:shadow-md transition-all duration-200 overflow-hidden group"
				>
					<div class="p-5">
						<div class="flex items-center gap-2 mb-2">
							<h3 class="text-lg font-semibold text-gray-900 group-hover:text-blue-600 transition-colors">
								{spot.name}
							</h3>
							{#if spot.rock_type && spot.rock_type !== 'unknown'}
								<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-stone-100 text-stone-700 capitalize shrink-0">{spot.rock_type}</span>
							{/if}
						</div>

						<div class="flex flex-wrap gap-1.5 mb-2">
							{#each (spot.climbing_types ?? []) as ct}
								<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-50 text-blue-700 capitalize">{ct}</span>
							{/each}
						</div>

						<p class="text-xs text-gray-400">{spot.latitude.toFixed(4)}, {spot.longitude.toFixed(4)}</p>
					</div>
					<div class="h-0.5 bg-gradient-to-r from-blue-500 to-green-500 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300"></div>
				</a>
			{/each}
		</div>
	{/if}
</div>
