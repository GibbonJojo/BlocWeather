<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import type { AdminSpot, Country, AdminSubregion, CreateSpotData } from '$lib/api/client';

	export let token: string;

	const ROCK_TYPES = ['granite','sandstone','limestone','basalt','gneiss','quartzite','volcanic','conglomerate','gritstone','unknown'];
	const CLIMBING_TYPES = ['boulder', 'climbing', 'multipitch'];

	let spots: AdminSpot[] = [];
	let countries: Country[] = [];
	let subregions: AdminSubregion[] = [];
	let loading = true;
	let error = '';

	let editingId: string | null = null;
	let showForm = false;
	let saving = false;
	let formError = '';

	let form: CreateSpotData & { id?: string } = emptyForm();

	function emptyForm(): CreateSpotData {
		return { name: '', latitude: 0, longitude: 0, country_id: '', subregion_id: undefined,
			rock_type: 'unknown', description: undefined, climbing_types: [] };
	}

	$: filteredSubregions = subregions.filter(s => s.country_id === form.country_id);

	onMount(async () => {
		try {
			[spots, countries, subregions] = await Promise.all([
				api.adminListSpots(token),
				api.getCountries(),
				api.adminListSubregions(token),
			]);
		} catch (e) {
			error = 'Failed to load data.';
		} finally {
			loading = false;
		}
	});

	function startAdd() {
		editingId = null;
		form = emptyForm();
		showForm = true;
		formError = '';
	}

	function startEdit(spot: AdminSpot) {
		editingId = spot.id;
		form = {
			name: spot.name,
			latitude: spot.latitude,
			longitude: spot.longitude,
			country_id: spot.country_id,
			subregion_id: spot.subregion_id,
			rock_type: spot.rock_type ?? 'unknown',
			description: spot.description,
			climbing_types: spot.climbing_types ?? [],
		};
		showForm = true;
		formError = '';
	}

	async function save() {
		if (!form.name || !form.country_id) { formError = 'Name and country are required.'; return; }
		saving = true; formError = '';
		try {
			if (editingId) {
				const updated = await api.adminUpdateSpot(token, editingId, form);
				spots = spots.map(s => s.id === editingId ? { ...s, ...form, ...updated } : s);
			} else {
				const created = await api.adminCreateSpot(token, form);
				// Refresh list to get full context (country_name etc.)
				spots = await api.adminListSpots(token);
				void created;
			}
			showForm = false;
			editingId = null;
		} catch (e) {
			formError = 'Save failed. Check coordinates and required fields.';
		} finally {
			saving = false;
		}
	}

	async function remove(id: string, name: string) {
		if (!confirm(`Delete "${name}"? This also deletes all its weather data.`)) return;
		try {
			await api.adminDeleteSpot(token, id);
			spots = spots.filter(s => s.id !== id);
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
			<span class="text-sm text-gray-500">{spots.length} spots</span>
			<button on:click={startAdd}
				class="px-3 py-1.5 bg-gray-900 text-white text-sm rounded-lg hover:bg-gray-700 transition-colors cursor-pointer">
				+ Add spot
			</button>
		</div>

		{#if showForm}
			<div class="border border-gray-200 rounded-lg p-4 space-y-3 bg-gray-50">
				<h3 class="font-medium text-gray-900">{editingId ? 'Edit spot' : 'New spot'}</h3>
				<div class="grid grid-cols-2 gap-3">
					<div class="col-span-2">
						<label class="text-xs text-gray-500">Name *</label>
						<input bind:value={form.name} class="input" placeholder="Spot name" />
					</div>
					<div>
						<label class="text-xs text-gray-500">Latitude *</label>
						<input bind:value={form.latitude} type="number" step="any" class="input" />
					</div>
					<div>
						<label class="text-xs text-gray-500">Longitude *</label>
						<input bind:value={form.longitude} type="number" step="any" class="input" />
					</div>
					<div>
						<label class="text-xs text-gray-500">Country *</label>
						<select bind:value={form.country_id} on:change={() => form.subregion_id = undefined} class="input">
							<option value="">— select —</option>
							{#each countries as c}
								<option value={c.id}>{c.name}</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="text-xs text-gray-500">Region</label>
						<select bind:value={form.subregion_id} class="input" disabled={!form.country_id}>
							<option value={undefined}>— none —</option>
							{#each filteredSubregions as s}
								<option value={s.id}>{s.name}</option>
							{/each}
						</select>
					</div>
					<div>
						<label class="text-xs text-gray-500">Rock type</label>
						<select bind:value={form.rock_type} class="input">
							{#each ROCK_TYPES as r}<option value={r}>{r}</option>{/each}
						</select>
					</div>
					<div class="col-span-2">
						<label class="text-xs text-gray-500">Climbing types</label>
						<div class="flex gap-3 mt-1">
							{#each CLIMBING_TYPES as ct}
								<label class="flex items-center gap-1.5 text-sm text-gray-700 cursor-pointer">
									<input type="checkbox" bind:group={form.climbing_types} value={ct} class="rounded" />
									{ct}
								</label>
							{/each}
						</div>
					</div>
					<div class="col-span-2">
						<label class="text-xs text-gray-500">Description</label>
						<textarea bind:value={form.description} class="input h-16 resize-none" placeholder="optional"></textarea>
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

		<div class="overflow-x-auto">
			<table class="w-full text-sm text-left">
				<thead>
					<tr class="border-b border-gray-200 text-gray-500 text-xs">
						<th class="pb-2 pr-4 font-medium">Name</th>
						<th class="pb-2 pr-4 font-medium">Country / Region</th>
						<th class="pb-2 pr-4 font-medium">Rock</th>
						<th class="pb-2 font-medium">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each spots as spot (spot.id)}
						<tr class="border-b border-gray-100 hover:bg-gray-50">
							<td class="py-2 pr-4 font-medium text-gray-900">{spot.name}</td>
							<td class="py-2 pr-4 text-gray-600">
								{spot.country_name}{spot.subregion_name ? ` / ${spot.subregion_name}` : ''}
							</td>
							<td class="py-2 pr-4 text-gray-500">{spot.rock_type ?? '—'}</td>
								<td class="py-2 flex gap-2">
								<button on:click={() => startEdit(spot)} class="text-blue-600 hover:underline cursor-pointer text-xs">Edit</button>
								<button on:click={() => remove(spot.id, spot.name)} class="text-red-500 hover:underline cursor-pointer text-xs">Delete</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	@reference "tailwindcss";
	.input {
		@apply w-full border border-gray-300 rounded-md px-2.5 py-1.5 text-sm text-gray-900 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500;
	}
</style>
