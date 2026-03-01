<script lang="ts">
	import '../app.css';
	import SearchBar from '$lib/components/SearchBar.svelte';
	import { page } from '$app/stores';

	$: isEmbed = $page.url.pathname.startsWith('/embed');
</script>

{#if isEmbed}
	<slot />
{:else}
	<div class="min-h-screen bg-gray-50">
		<!-- Header -->
		<header class="bg-white border-b border-gray-200 sticky top-0 z-50">
			<nav class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div class="flex items-center gap-4 h-16">
					<a href="/" class="text-xl font-bold text-blue-600 shrink-0">🧗 BlocWeather</a>
					<div class="flex-1">
						<SearchBar />
					</div>
				</div>
			</nav>
		</header>

		<!-- Main content -->
		<main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
			<slot />
		</main>

		<!-- Disclaimer footer -->
		<footer class="border-t border-gray-200 mt-8 py-6">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<p class="text-xs text-gray-400 text-center">
					Rock wetness is an estimation based on weather data and may not reflect actual conditions.
					Always assess conditions yourself before climbing.
					Never climb on wet sandstone — it damages the rock permanently.
				</p>
			</div>
		</footer>
	</div>
{/if}
