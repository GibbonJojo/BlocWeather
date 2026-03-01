<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import type { AreaSuggestion } from '$lib/api/client';

	export let token: string;

	let suggestions: AreaSuggestion[] = [];
	let loading = true;
	let error = '';

	onMount(async () => {
		try {
			suggestions = await api.adminListSuggestions(token);
		} catch {
			error = 'Failed to load suggestions.';
		} finally {
			loading = false;
		}
	});

	async function remove(id: string, name: string) {
		if (!confirm(`Delete suggestion "${name}"?`)) return;
		try {
			await api.adminDeleteSuggestion(token, id);
			suggestions = suggestions.filter(s => s.id !== id);
		} catch {
			alert('Delete failed.');
		}
	}

	async function removeAll() {
		if (!confirm(`Delete all ${suggestions.length} suggestion(s)? This cannot be undone.`)) return;
		try {
			await api.adminDeleteAllSuggestions(token);
			suggestions = [];
		} catch {
			alert('Delete failed.');
		}
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleDateString(undefined, { day: '2-digit', month: 'short', year: 'numeric' });
	}
</script>

<div class="space-y-4">
	{#if error}
		<p class="text-red-600 text-sm">{error}</p>
	{:else if loading}
		<p class="text-gray-400 text-sm">Loading…</p>
	{:else}
		<div class="flex justify-between items-center">
			<span class="text-sm text-gray-500">{suggestions.length} suggestion{suggestions.length !== 1 ? 's' : ''}</span>
			{#if suggestions.length > 0}
				<button
					on:click={removeAll}
					class="px-3 py-1.5 text-sm border border-red-300 text-red-600 rounded-lg hover:bg-red-50 transition-colors cursor-pointer"
				>
					Delete all
				</button>
			{/if}
		</div>

		{#if suggestions.length === 0}
			<p class="text-sm text-gray-400 py-4">No suggestions yet.</p>
		{:else}
			<table class="w-full text-sm text-left">
				<thead>
					<tr class="border-b border-gray-200 text-gray-500 text-xs">
						<th class="pb-2 pr-4 font-medium">Area name</th>
						<th class="pb-2 pr-4 font-medium">Country</th>
						<th class="pb-2 pr-4 font-medium">Submitted</th>
						<th class="pb-2 font-medium">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each suggestions as s (s.id)}
						<tr class="border-b border-gray-100 hover:bg-gray-50">
							<td class="py-2 pr-4 font-medium text-gray-900">{s.name}</td>
							<td class="py-2 pr-4 text-gray-500">{s.country}</td>
							<td class="py-2 pr-4 text-gray-400 text-xs">{formatDate(s.created_at)}</td>
							<td class="py-2">
								<button on:click={() => remove(s.id, s.name)} class="text-red-500 hover:underline cursor-pointer text-xs">Delete</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	{/if}
</div>
