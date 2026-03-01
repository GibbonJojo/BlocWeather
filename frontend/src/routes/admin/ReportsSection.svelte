<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import type { AdminReport } from '$lib/api/client';

	export let token: string;

	let reports: AdminReport[] = [];
	let loading = true;
	let error = '';

	type SortKey = 'spot_name' | 'observed_at' | 'status' | 'match' | 'reported_at';
	let sortKey: SortKey = 'reported_at';
	let sortDir: 1 | -1 = -1; // -1 = desc, 1 = asc

	// Reported status → expected saturation range [min, max]
	const EXPECTED: Record<string, [number, number]> = {
		dry:        [0.00, 0.20],
		some_wet:   [0.20, 0.50],
		mostly_wet: [0.50, 0.80],
		wet:        [0.80, 1.00],
	};

	const STATUS_LABEL: Record<string, string> = {
		dry: 'Dry', some_wet: 'Some wet', mostly_wet: 'Mostly wet', wet: 'Wet',
	};

	// 0 = no data, 1 = mismatch, 2 = partial, 3 = match (for sort)
	function matchScore(r: AdminReport): number {
		if (r.calc_min_saturation == null || r.calc_max_saturation == null) return 0;
		const [expMin, expMax] = EXPECTED[r.status] ?? [0, 1];
		const overlap = Math.max(0, Math.min(r.calc_max_saturation, expMax) - Math.max(r.calc_min_saturation, expMin));
		const coverage = overlap / (expMax - expMin);
		if (coverage > 0.5) return 3;
		if (coverage > 0)   return 2;
		return 1;
	}

	function matchLabel(r: AdminReport): { text: string; cls: string } {
		const score = matchScore(r);
		if (score === 0) return { text: 'No data', cls: 'text-gray-400' };
		if (score === 3) return { text: 'Match',   cls: 'text-green-600' };
		if (score === 2) return { text: 'Partial', cls: 'text-amber-600' };
		return { text: 'Mismatch', cls: 'text-red-600' };
	}

	function satPct(v?: number) {
		return v != null ? `${(v * 100).toFixed(0)}%` : '—';
	}

	function fmtDate(iso: string) {
		return new Date(iso).toLocaleString(undefined, {
			month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
		});
	}

	function sortValue(r: AdminReport): string | number {
		if (sortKey === 'spot_name')    return r.spot_name.toLowerCase();
		if (sortKey === 'observed_at')  return r.observed_at;
		if (sortKey === 'reported_at')  return r.reported_at;
		if (sortKey === 'status')       return r.status;
		if (sortKey === 'match')        return matchScore(r);
		return '';
	}

	$: sorted = [...reports].sort((a, b) => {
		const av = sortValue(a), bv = sortValue(b);
		return av < bv ? -sortDir : av > bv ? sortDir : 0;
	});

	function toggleSort(key: SortKey) {
		if (sortKey === key) { sortDir = sortDir === 1 ? -1 : 1; }
		else { sortKey = key; sortDir = key === 'reported_at' ? -1 : 1; }
	}

	function arrow(key: SortKey) {
		if (sortKey !== key) return '';
		return sortDir === 1 ? ' ↑' : ' ↓';
	}

	async function remove(id: string) {
		if (!confirm('Delete this report?')) return;
		try {
			await api.adminDeleteReport(token, id);
			reports = reports.filter(r => r.id !== id);
		} catch {
			alert('Delete failed.');
		}
	}

	onMount(async () => {
		try {
			reports = await api.adminListReports(token);
		} catch {
			error = 'Failed to load reports.';
		} finally {
			loading = false;
		}
	});
</script>

<div class="space-y-4">
	{#if error}
		<p class="text-red-600 text-sm">{error}</p>
	{:else if loading}
		<p class="text-gray-400 text-sm">Loading…</p>
	{:else}
		<div class="flex items-center justify-between">
			<span class="text-sm text-gray-500">{reports.length} reports</span>
			<span class="text-xs text-gray-400">Algorithm range compared to reported status</span>
		</div>

		{#if reports.length === 0}
			<p class="text-gray-400 text-sm">No reports yet.</p>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full text-sm text-left">
					<thead>
						<tr class="border-b border-gray-200 text-gray-500 text-xs">
							<th class="pb-2 pr-4 font-medium">
								<button class="hover:text-gray-900 cursor-pointer" on:click={() => toggleSort('spot_name')}>
									Spot{arrow('spot_name')}
								</button>
							</th>
							<th class="pb-2 pr-4 font-medium">
								<button class="hover:text-gray-900 cursor-pointer" on:click={() => toggleSort('observed_at')}>
									Observed at{arrow('observed_at')}
								</button>
							</th>
							<th class="pb-2 pr-4 font-medium">
								<button class="hover:text-gray-900 cursor-pointer" on:click={() => toggleSort('status')}>
									Reported{arrow('status')}
								</button>
							</th>
							<th class="pb-2 pr-4 font-medium">Algo min–max sat.</th>
							<th class="pb-2 pr-4 font-medium">
								<button class="hover:text-gray-900 cursor-pointer" on:click={() => toggleSort('match')}>
									Match{arrow('match')}
								</button>
							</th>
							<th class="pb-2 font-medium">
								<button class="hover:text-gray-900 cursor-pointer" on:click={() => toggleSort('reported_at')}>
									Submitted{arrow('reported_at')}
								</button>
							</th>
							<th class="pb-2"></th>
						</tr>
					</thead>
					<tbody>
						{#each sorted as r (r.id)}
							{@const m = matchLabel(r)}
							<tr class="border-b border-gray-100 hover:bg-gray-50">
								<td class="py-2 pr-4 font-medium text-gray-900">{r.spot_name}</td>
								<td class="py-2 pr-4 text-gray-600">{fmtDate(r.observed_at)}</td>
								<td class="py-2 pr-4 text-gray-700">{STATUS_LABEL[r.status] ?? r.status}</td>
								<td class="py-2 pr-4 text-gray-500 font-mono text-xs">
									{satPct(r.calc_min_saturation)} – {satPct(r.calc_max_saturation)}
								</td>
								<td class="py-2 pr-4 font-medium {m.cls}">{m.text}</td>
								<td class="py-2 pr-4 text-gray-500">{fmtDate(r.reported_at)}</td>
								<td class="py-2">
									<button on:click={() => remove(r.id)} class="text-red-500 hover:underline cursor-pointer text-xs">Delete</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	{/if}
</div>
