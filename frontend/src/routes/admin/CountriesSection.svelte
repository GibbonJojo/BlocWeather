<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import type { Country } from '$lib/api/client';

	export let token: string;

	let countries: Country[] = [];
	let loading = true;
	let error = '';

	let editingId: string | null = null;
	let showForm = false;
	let saving = false;
	let formError = '';
	let formName = '';
	let formCode = '';

	function sortCountries(list: Country[]): Country[] {
		return [...list].sort((a, b) => a.name.localeCompare(b.name));
	}

	onMount(async () => {
		try {
			countries = sortCountries(await api.getCountries());
		} catch {
			error = 'Failed to load countries.';
		} finally {
			loading = false;
		}
	});

	function startAdd() {
		editingId = null; formName = ''; formCode = '';
		showForm = true; formError = '';
	}

	function startEdit(c: Country) {
		editingId = c.id; formName = c.name; formCode = c.code;
		showForm = true; formError = '';
	}

	async function save() {
		if (!formName || !formCode) { formError = 'Name and code are required.'; return; }
		saving = true; formError = '';
		try {
			if (editingId) {
				await api.adminUpdateCountry(token, editingId, { name: formName, code: formCode });
				countries = sortCountries(countries.map(c => c.id === editingId ? { ...c, name: formName, code: formCode } : c));
			} else {
				await api.adminCreateCountry(token, { name: formName, code: formCode });
				countries = sortCountries(await api.getCountries());
			}
			showForm = false; editingId = null;
		} catch {
			formError = 'Save failed.';
		} finally {
			saving = false;
		}
	}

	async function remove(id: string, name: string) {
		if (!confirm(`Delete "${name}"? This also deletes all subregions and spots.`)) return;
		try {
			await api.adminDeleteCountry(token, id);
			countries = countries.filter(c => c.id !== id);
		} catch {
			alert('Delete failed.');
		}
	}
</script>

<div class="space-y-4">
	{#if error}
		<p class="text-red-600 text-sm">{error}</p>
	{:else if loading}
		<p class="text-gray-400 text-sm">Loading…</p>
	{:else}
		<div class="flex justify-between items-center">
			<span class="text-sm text-gray-500">{countries.length} countries</span>
			<button on:click={startAdd}
				class="px-3 py-1.5 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 transition-colors cursor-pointer">
				+ Add country
			</button>
		</div>

		{#if showForm}
			<div class="border border-gray-200 rounded-lg p-4 space-y-3 bg-gray-50">
				<h3 class="font-medium text-gray-900">{editingId ? 'Edit country' : 'New country'}</h3>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="text-xs text-gray-500">Name *</label>
						<input bind:value={formName} class="input" placeholder="Germany" />
					</div>
					<div>
						<label class="text-xs text-gray-500">ISO code * (2 letters)</label>
						<input bind:value={formCode} class="input" placeholder="DE" maxlength="2" />
					</div>
				</div>
				{#if formError}<p class="text-red-600 text-xs">{formError}</p>{/if}
				<div class="flex gap-2">
					<button on:click={() => showForm = false} class="px-3 py-1.5 border border-gray-300 text-sm rounded-lg text-gray-600 hover:bg-gray-100 cursor-pointer">Cancel</button>
					<button on:click={save} disabled={saving} class="px-3 py-1.5 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 disabled:opacity-40 cursor-pointer">
						{saving ? 'Saving…' : 'Save'}
					</button>
				</div>
			</div>
		{/if}

		<table class="w-full text-sm text-left">
			<thead>
				<tr class="border-b border-gray-200 text-gray-500 text-xs">
					<th class="pb-2 pr-4 font-medium">Name</th>
					<th class="pb-2 pr-4 font-medium">Code</th>
					<th class="pb-2 pr-4 font-medium">Spots</th>
					<th class="pb-2 font-medium">Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each countries as c (c.id)}
					<tr class="border-b border-gray-100 hover:bg-gray-50">
						<td class="py-2 pr-4 font-medium text-gray-900">{c.name}</td>
						<td class="py-2 pr-4 text-gray-500">{c.code}</td>
						<td class="py-2 pr-4 text-gray-500">{c.spot_count}</td>
						<td class="py-2 flex gap-2">
							<button on:click={() => startEdit(c)} class="text-blue-600 hover:underline cursor-pointer text-xs">Edit</button>
							<button on:click={() => remove(c.id, c.name)} class="text-red-500 hover:underline cursor-pointer text-xs">Delete</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</div>

<style>
	@reference "tailwindcss";
	.input {
		@apply w-full border border-gray-300 rounded-md px-2.5 py-1.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500;
	}
</style>
