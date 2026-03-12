<script lang="ts">
	import { api } from '$lib/api/client';
	import type { ConditionStatus } from '$lib/api/client';
	import type { PageData } from './$types';

	export let data: PageData;

	const STATUS_LABELS: Record<string, string> = {
		dry:        'Dry',
		some_wet:   'Some wet',
		mostly_wet: 'Mostly wet',
		wet:        'Wet',
	};

	const STATUS_STYLES: Record<string, string> = {
		dry:        'border-green-500 text-green-700 bg-green-50',
		some_wet:   'border-sky-300 text-sky-700 bg-sky-50',
		mostly_wet: 'border-orange-500 text-orange-700 bg-orange-50',
		wet:        'border-blue-500 text-blue-700 bg-blue-50',
	};

	const STATUS_ACTIVE: Record<string, string> = {
		dry:        'bg-green-500 text-white border-green-500',
		some_wet:   'bg-sky-400 text-white border-sky-400',
		mostly_wet: 'bg-orange-500 text-white border-orange-500',
		wet:        'bg-blue-500 text-white border-blue-500',
	};

	let selectedStatus = '';
	let observedTime = '';
	let comment = '';
	let submitting = false;
	let success = false;
	let submitError = '';

	function maxDatetime(): string {
		const now = new Date(Date.now() - 60_000);
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
	}

	// Default time to current hour
	const now = new Date();
	now.setMinutes(0, 0, 0);
	const pad = (n: number) => String(n).padStart(2, '0');
	observedTime = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:00`;

	async function submit() {
		if (!selectedStatus || !observedTime) return;
		submitting = true;
		submitError = '';
		try {
			await api.submitReport(data.spot.id, new Date(observedTime).toISOString(), selectedStatus as ConditionStatus, comment);
			success = true;
		} catch {
			submitError = 'Could not submit. Please try again.';
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>{data.spot.name} – Report – BlocWeather</title>
</svelte:head>

<div style="font-family: system-ui, sans-serif;" class="bg-white p-4 min-h-screen">
	<div class="flex items-baseline justify-between mb-4">
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

	{#if success}
		<div class="flex items-center gap-2 text-green-700 bg-green-50 border border-green-200 rounded-lg p-3 text-sm">
			<svg class="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
			</svg>
			Report submitted. Thank you!
		</div>
	{:else}
		<div class="space-y-4">
			<div class="space-y-1.5">
				<label for="observed-time" class="block text-xs font-medium text-gray-600">When did you observe this?</label>
				<input
					id="observed-time"
					type="datetime-local"
					step="3600"
					max={maxDatetime()}
					bind:value={observedTime}
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
				/>
			</div>

			<div class="space-y-1.5">
				<p class="text-xs font-medium text-gray-600">Rock conditions</p>
				<div class="grid grid-cols-2 gap-2">
					{#each Object.entries(STATUS_LABELS) as [status, label]}
						<button
							on:click={() => { selectedStatus = status; }}
							class="py-2 px-3 rounded-lg border text-sm font-medium transition-all cursor-pointer {selectedStatus === status ? STATUS_ACTIVE[status] : STATUS_STYLES[status]}"
						>
							{label}
						</button>
					{/each}
				</div>
			</div>

			<div class="space-y-1.5">
				<label for="comment" class="block text-xs font-medium text-gray-600">Comment <span class="text-gray-400">(optional)</span></label>
				<textarea
					id="comment"
					rows="2"
					maxlength="500"
					placeholder="e.g. north face still damp"
					bind:value={comment}
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
				></textarea>
			</div>

			{#if submitError}
				<p class="text-xs text-red-600">{submitError}</p>
			{/if}

			<button
				on:click={submit}
				disabled={!selectedStatus || !observedTime || submitting}
				class="w-full py-2 rounded-lg bg-gray-900 text-sm text-white font-medium hover:bg-gray-700 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
			>
				{submitting ? 'Submitting…' : 'Submit report'}
			</button>
		</div>
	{/if}
</div>
