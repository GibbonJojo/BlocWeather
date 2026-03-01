<script lang="ts">
	import type { Country } from '$lib/api/client';

	export let country: Country;
</script>

<a
	href="/countries/{country.id}"
	class="block bg-white rounded-lg shadow-md hover:shadow-xl transition-shadow duration-300 overflow-hidden group"
>
	<div class="p-6">
		<div class="flex items-center justify-between mb-4">
			<h3 class="text-2xl font-bold text-gray-900 group-hover:text-blue-600 transition-colors">
				{country.name}
			</h3>
			<span class="text-3xl">{country.code === 'US' ? '🇺🇸' : country.code === 'FR' ? '🇫🇷' : country.code === 'DE' ? '🇩🇪' : country.code === 'IT' ? '🇮🇹' : country.code === 'ES' ? '🇪🇸' : country.code === 'CH' ? '🇨🇭' : '🌍'}</span>
		</div>
		<div class="flex items-center text-gray-600">
			<span class="text-sm">
				{country.spot_count} {country.spot_count === 1 ? 'spot' : 'spots'}
			</span>
		</div>
	</div>
	<div class="h-1 bg-gradient-to-r from-blue-500 to-green-500 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-300"></div>
</a>
