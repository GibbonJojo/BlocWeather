<script lang="ts">
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>Climbing Spots - BlocWeather</title>
</svelte:head>

<div class="space-y-6">
	<!-- Back button -->
	<button
		on:click={() => window.history.back()}
		class="inline-flex items-center text-blue-600 hover:text-blue-700 transition-colors cursor-pointer"
	>
		<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		Back to Subregions
	</button>

	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold text-gray-900">Climbing Spots</h1>
		<p class="text-gray-600 mt-2">{data.spots.length} {data.spots.length === 1 ? 'spot' : 'spots'} available</p>
	</div>

	<!-- Spots grid -->
	{#if data.spots.length === 0}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
			<span class="text-4xl block mb-2">🧗</span>
			<p class="text-yellow-800 font-medium">No climbing spots found</p>
			<p class="text-yellow-600 text-sm mt-2">Check back soon as we add more locations!</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			{#each data.spots as spot (spot.id)}
				<a
					href="/spots/{spot.id}"
					class="block bg-white rounded-lg shadow-md hover:shadow-xl transition-all duration-300 overflow-hidden group"
				>
					<div class="p-6">
						<div class="flex items-start justify-between mb-3">
							<h3 class="text-xl font-bold text-gray-900 group-hover:text-blue-600 transition-colors flex-1">
								{spot.name}
							</h3>
							</div>

						{#if spot.description}
							<p class="text-gray-600 text-sm mb-4 line-clamp-2">{spot.description}</p>
						{/if}

						<div class="space-y-2 text-sm text-gray-500">
							{#if spot.elevation_meters}
								<div class="flex items-center">
									<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
									</svg>
									{spot.elevation_meters}m elevation
								</div>
							{/if}

														<div class="flex items-center">
								<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" />
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" />
								</svg>
								{spot.latitude.toFixed(4)}, {spot.longitude.toFixed(4)}
							</div>
						</div>
					</div>
					<div class="h-1 bg-gradient-to-r from-blue-500 to-green-500 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300"></div>
				</a>
			{/each}
		</div>
	{/if}
</div>
