<script lang="ts">
	import CountryCard from '$lib/components/CountryCard.svelte';
	import SpotMap from '$lib/components/SpotMap.svelte';
	import type { PageData } from './$types';

	export let data: PageData;

	let mapExpanded = false;

	function toggleMap() {
		mapExpanded = !mapExpanded;
	}
</script>

<svelte:head>
	<title>BlocWeather - Climbing Weather Conditions</title>
	<meta name="description" content="Real-time climbing weather conditions for bouldering and climbing spots worldwide" />
</svelte:head>

<div class="space-y-8">
	<!-- Hero Section -->
	<div class="text-center space-y-4">
		<h1 class="text-4xl md:text-5xl font-bold text-gray-900">
			Find Perfect Climbing Conditions
		</h1>
		<p class="text-lg text-gray-600 max-w-2xl mx-auto">
			Real-time climbing weather conditions for bouldering and climbing spots worldwide.
		</p>
	</div>

	<!-- Map Section (Collapsible) -->
	<div class="bg-white rounded-lg shadow-lg overflow-hidden">
		<button
			on:click={toggleMap}
			class="w-full px-6 py-4 flex items-center justify-between hover:bg-gray-50 transition-colors"
		>
			<div class="flex items-center space-x-3">
				<span class="text-2xl">🗺️</span>
				<h2 class="text-xl font-semibold text-gray-900">Interactive Map</h2>
			</div>
			<svg
				class="w-6 h-6 text-gray-400 transform transition-transform {mapExpanded ? 'rotate-180' : ''}"
				fill="none"
				stroke="currentColor"
				viewBox="0 0 24 24"
			>
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>

		{#if mapExpanded}
			<div class="h-[500px] p-4">
				<SpotMap />
			</div>
		{/if}
	</div>

	<!-- Countries Section -->
	<div class="space-y-6">
		<div class="flex items-center justify-between">
			<h2 class="text-2xl font-bold text-gray-900">Browse by Country</h2>
			<span class="text-sm text-gray-500">{data.countries.length} countries</span>
		</div>

		{#if data.error}
			<div class="bg-red-50 border border-red-200 rounded-lg p-4">
				<p class="text-red-800">{data.error}</p>
			</div>
		{:else if data.countries.length === 0}
			<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
				<span class="text-4xl block mb-2">📍</span>
				<p class="text-yellow-800 font-medium">No climbing spots added yet</p>
				<p class="text-yellow-600 text-sm mt-2">Check back soon as we add more locations!</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
				{#each data.countries as country (country.id)}
					<CountryCard {country} />
				{/each}
			</div>
		{/if}
	</div>
</div>
