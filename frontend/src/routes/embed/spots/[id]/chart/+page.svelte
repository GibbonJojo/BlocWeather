<script lang="ts">
	import WeatherChart from '$lib/components/WeatherChart.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.spot.name} – BlocWeather</title>
</svelte:head>

<div class="bg-white p-4" style="font-family: system-ui, sans-serif;">
	<div class="flex items-baseline justify-between mb-3">
		<span class="font-semibold text-gray-900 text-sm">{data.spot.name}</span>
		<a
			href="/spots/{data.spot.id}"
			target="_blank"
			rel="noopener"
			class="text-xs text-blue-600 hover:underline shrink-0 ml-4"
		>
			BlocWeather ↗
		</a>
	</div>
	<WeatherChart weather={data.weather} conditions={data.conditions} initialActive={data.initialActive} />
</div>
