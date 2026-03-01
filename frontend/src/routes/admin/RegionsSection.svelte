<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import type { AdminSubregion, Country } from '$lib/api/client';

	export let token: string;

	let regions: AdminSubregion[] = [];
	let countries: Country[] = [];
	let loading = true;
	let error = '';

	let editingId: string | null = null;
	let showForm = false;
	let saving = false;
	let formError = '';
	let formName = '';
	let formCountryId = '';

	function sortRegions(list: AdminSubregion[]): AdminSubregion[] {
		return [...list].sort((a, b) =>
			a.country_name.localeCompare(b.country_name) || a.name.localeCompare(b.name)
		);
	}

	onMount(async () => {
		try {
			const [r, c] = await Promise.all([
				api.adminListSubregions(token),
				api.getCountries(),
			]);
			regions = sortRegions(r);
			countries = [...c].sort((a, b) => a.name.localeCompare(b.name));
		} catch {
			error = 'Failed to load data.';
		} finally {
			loading = false;
		}
	});

	function startAdd() {
		editingId = null; formName = ''; formCountryId = '';
		showForm = true; formError = '';
	}

	function startEdit(r: AdminSubregion) {
		editingId = r.id; formName = r.name; formCountryId = r.country_id;
		showForm = true; formError = '';
	}

	async function save() {
		if (!formName || !formCountryId) { formError = 'Name and country are required.'; return; }
		saving = true; formError = '';
		try {
			if (editingId) {
				await api.adminUpdateSubregion(token, editingId, { name: formName });
				regions = sortRegions(regions.map(r => r.id === editingId ? { ...r, name: formName } : r));
			} else {
				await api.adminCreateSubregion(token, { name: formName, country_id: formCountryId });
				regions = sortRegions(await api.adminListSubregions(token));
			}
			showForm = false; editingId = null;
		} catch {
			formError = 'Save failed.';
		} finally {
			saving = false;
		}
	}

	async function remove(id: string, name: string) {
		if (!confirm(`Delete region "${name}"?`)) return;
		try {
			await api.adminDeleteSubregion(token, id);
			regions = regions.filter(r => r.id !== id);
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
			<span class="text-sm text-gray-500">{regions.length} regions</span>
			<button on:click={startAdd}
				class="px-3 py-1.5 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 transition-colors cursor-pointer">
				+ Add region
			</button>
		</div>

		{#if showForm}
			<div class="border border-gray-200 rounded-lg p-4 space-y-3 bg-gray-50">
				<h3 class="font-medium text-gray-900">{editingId ? 'Edit region' : 'New region'}</h3>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="text-xs text-gray-500">Name *</label>
						<input bind:value={formName} class="input" placeholder="Bayern" />
					</div>
					<div>
						<label class="text-xs text-gray-500">Country *</label>
						<select bind:value={formCountryId} class="input" disabled={!!editingId}>
							<option value="">— select —</option>
							{#each countries as c}
								<option value={c.id}>{c.name}</option>
							{/each}
						</select>
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
					<th class="pb-2 pr-4 font-medium">Country</th>
					<th class="pb-2 pr-4 font-medium">Spots</th>
					<th class="pb-2 font-medium">Actions</th>
				</tr>
			</thead>
			<tbody>
				{#each regions as r (r.id)}
					<tr class="border-b border-gray-100 hover:bg-gray-50">
						<td class="py-2 pr-4 font-medium text-gray-900">{r.name}</td>
						<td class="py-2 pr-4 text-gray-600">{r.country_name}</td>
						<td class="py-2 pr-4 text-gray-500">{r.spot_count}</td>
						<td class="py-2 flex gap-2">
							<button on:click={() => startEdit(r)} class="text-blue-600 hover:underline cursor-pointer text-xs">Edit</button>
							<button on:click={() => remove(r.id, r.name)} class="text-red-500 hover:underline cursor-pointer text-xs">Delete</button>
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
