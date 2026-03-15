<script lang="ts">
	import WeatherChart from '$lib/components/WeatherChart.svelte';
	import type { PageData } from './$types';
	import { onMount } from 'svelte';

	export let data: PageData;

	let activeTab: 'current' | 'forecast' = 'current';
	const forecast14InitialActive = { rockTemp: false, sunshine: false, wind: false };
	let forecast14Compact = false;
	onMount(() => { forecast14Compact = window.innerWidth >= 640; });
</script>

<svelte:head>
	<title>{data.spot.name} – BlocWeather</title>
</svelte:head>

<div class="bg-white p-4" style="font-family: system-ui, sans-serif;">
	<div class="flex items-baseline justify-between mb-3">
		<span class="font-semibold text-gray-900 text-sm">{data.spot.name}</span>
		<a
			href="/{data.spot.country?.slug ?? ''}/{data.spot.subregion?.slug ?? '-'}/{data.spot.slug}"
			target="_blank"
			rel="noopener"
			class="text-xs text-blue-600 hover:underline shrink-0 ml-4"
		>
			BlocWeather ↗
		</a>
	</div>

	<div class="flex gap-4 border-b border-gray-200 mb-3">
		<button on:click={() => activeTab = 'current'} class="pb-1.5 text-xs font-medium border-b-2 transition-colors cursor-pointer {activeTab === 'current' ? 'border-gray-900 text-gray-900' : 'border-transparent text-gray-400'}">Now / 5-day</button>
		<button on:click={() => activeTab = 'forecast'} class="pb-1.5 text-xs font-medium border-b-2 transition-colors cursor-pointer {activeTab === 'forecast' ? 'border-gray-900 text-gray-900' : 'border-transparent text-gray-400'}">14-day</button>
	</div>

	{#if activeTab === 'current'}
		<WeatherChart weather={data.weather} conditions={data.conditions} initialActive={data.initialActive} />
	{:else}
		<p class="text-xs text-gray-400 mb-3">Forecast accuracy drops sharply after 3 days. Rough trend only.</p>
		<WeatherChart weather={data.forecast14Weather} conditions={data.forecast14Conditions} initialActive={forecast14InitialActive} compact={forecast14Compact} />
	{/if}
</div>
