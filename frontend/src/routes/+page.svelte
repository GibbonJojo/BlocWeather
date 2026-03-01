<script lang="ts">
	import CountryCard from '$lib/components/CountryCard.svelte';
	import SpotMap from '$lib/components/SpotMap.svelte';
	import { api } from '$lib/api/client';
	import type { PageData } from './$types';

	export let data: PageData;

	let mapExpanded = false;

	function toggleMap() {
		mapExpanded = !mapExpanded;
	}

	// Suggest area modal
	let showSuggestModal = false;
	let suggestName = '';
	let suggestCountry = '';
	let suggestSubmitting = false;
	let suggestError = '';
	let suggestSuccess = false;

	async function submitSuggestion() {
		if (!suggestName.trim() || !suggestCountry.trim()) {
			suggestError = 'Please fill in both fields.';
			return;
		}
		suggestSubmitting = true;
		suggestError = '';
		try {
			await api.submitAreaSuggestion(suggestName.trim(), suggestCountry.trim());
			suggestSuccess = true;
		} catch {
			suggestError = 'Something went wrong. Please try again.';
		} finally {
			suggestSubmitting = false;
		}
	}

	function closeSuggestModal() {
		showSuggestModal = false;
		suggestName = '';
		suggestCountry = '';
		suggestError = '';
		suggestSuccess = false;
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
			<div class="flex items-center gap-3">
				<span class="text-sm text-gray-500">{data.countries.length} countries</span>
				<button
					on:click={() => showSuggestModal = true}
					class="px-3 py-1.5 text-sm border border-gray-300 rounded-lg text-gray-600 hover:bg-gray-100 transition-colors cursor-pointer"
				>
					+ Suggest an area
				</button>
			</div>
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

<!-- Suggest area modal -->
{#if showSuggestModal}
	<div
		class="fixed inset-0 bg-black/40 z-50 flex items-center justify-center p-4"
		role="dialog"
		aria-modal="true"
		on:click|self={closeSuggestModal}
		on:keydown={(e) => e.key === 'Escape' && closeSuggestModal()}
		tabindex="-1"
	>
		<div class="bg-white rounded-xl shadow-xl w-full max-w-md p-6 space-y-4">
			{#if suggestSuccess}
				<div class="text-center space-y-3 py-2">
					<span class="text-4xl block">🙌</span>
					<p class="font-semibold text-gray-900">Thanks for the suggestion!</p>
					<p class="text-sm text-gray-500">We'll look into adding it.</p>
					<button on:click={closeSuggestModal} class="mt-2 px-4 py-2 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 cursor-pointer">
						Close
					</button>
				</div>
			{:else}
				<h2 class="text-lg font-semibold text-gray-900">Suggest an area</h2>
				<p class="text-sm text-gray-500">Know a climbing area we don't have yet? Let us know.</p>
				<div class="space-y-3">
					<div>
						<label class="text-xs text-gray-500 block mb-1">Area name *</label>
						<input
							bind:value={suggestName}
							type="text"
							placeholder="e.g. Frankenjura"
							class="input"
						/>
					</div>
					<div>
						<label class="text-xs text-gray-500 block mb-1">Country *</label>
						<input
							bind:value={suggestCountry}
							type="text"
							placeholder="e.g. Germany"
							on:keydown={(e) => e.key === 'Enter' && submitSuggestion()}
							class="input"
						/>
					</div>
				</div>
				{#if suggestError}<p class="text-red-600 text-xs">{suggestError}</p>{/if}
				<div class="flex gap-2 justify-end">
					<button on:click={closeSuggestModal} class="px-3 py-1.5 border border-gray-300 text-sm rounded-lg text-gray-600 hover:bg-gray-100 cursor-pointer">
						Cancel
					</button>
					<button
						on:click={submitSuggestion}
						disabled={suggestSubmitting}
						class="px-3 py-1.5 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 disabled:opacity-40 cursor-pointer"
					>
						{suggestSubmitting ? 'Sending…' : 'Send suggestion'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	@reference "tailwindcss";
	.input {
		@apply w-full border border-gray-300 rounded-md px-2.5 py-1.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500;
	}
</style>
