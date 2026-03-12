<script lang="ts">
	import WeatherChart from '$lib/components/WeatherChart.svelte';
	import { api } from '$lib/api/client';
	import type { ConditionStatus } from '$lib/api/client';
	import type { PageData } from './$types';
	import { page } from '$app/stores';

	export let data: PageData;

	$: countrySlug = data.spot.country?.slug ?? '';
	$: regionSlug = data.spot.subregion?.slug ?? '-';
	$: backHref = `/${countrySlug}/${regionSlug}`;

	// ── Embed section ─────────────────────────────────────────────────────────
	let showEmbed = false;
	let copied = false;
	let copied2 = false;

	$: iframeSrc = `${$page.url.origin}/embed/${countrySlug}/${regionSlug}/${data.spot.slug}/chart`;
	$: iframeCode = `<iframe\n  src="${iframeSrc}"\n  width="100%"\n  height="420"\n  frameborder="0"\n  style="border-radius: 8px; border: 1px solid #e5e7eb;"\n></iframe>`;
	$: iframeReportSrc = `${$page.url.origin}/embed/${countrySlug}/${regionSlug}/${data.spot.slug}/report`;
	$: iframeReportCode = `<iframe\n  src="${iframeReportSrc}"\n  width="100%"\n  height="220"\n  frameborder="0"\n  style="border-radius: 8px; border: 1px solid #e5e7eb;"\n></iframe>`;

	async function copyEmbed() {
		await navigator.clipboard.writeText(iframeCode);
		copied = true;
		setTimeout(() => { copied = false; }, 2000);
	}

	async function copyEmbed2() {
		await navigator.clipboard.writeText(iframeReportCode);
		copied2 = true;
		setTimeout(() => { copied2 = false; }, 2000);
	}

	// ── Report modal ──────────────────────────────────────────────────────────
	let showModal = false;
	let selectedStatus = '';
	let observedTime = '';
	let comment = '';
	let submitting = false;
	let submitSuccess = false;
	let submitError = '';

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

	function openModal() {
		const now = new Date();
		now.setMinutes(0, 0, 0);
		const pad = (n: number) => String(n).padStart(2, '0');
		observedTime = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:00`;
		selectedStatus = '';
		comment = '';
		submitSuccess = false;
		submitError = '';
		showModal = true;
	}

	function maxDatetime(): string {
		const now = new Date(Date.now() - 60_000);
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
	}

	async function submitReport() {
		if (!selectedStatus || !observedTime) return;
		submitting = true;
		submitError = '';
		try {
			const observedAt = new Date(observedTime).toISOString();
			await api.submitReport(data.spot.id, observedAt, selectedStatus as ConditionStatus, comment);
			submitSuccess = true;
			setTimeout(() => { showModal = false; submitSuccess = false; }, 1800);
		} catch {
			submitError = 'Could not submit report. Please try again.';
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head>
	<title>{data.spot.name} - BlocWeather</title>
	<meta name="description" content="Climbing conditions for {data.spot.name}" />
</svelte:head>

<div class="space-y-4">
	<!-- Back button -->
	<a
		href={backHref}
		class="inline-flex items-center text-blue-600 hover:text-blue-700 transition-colors"
	>
		<svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
		</svg>
		{data.spot.subregion?.name ?? data.spot.country?.name ?? 'Back'}
	</a>

	<!-- Crag name + report button -->
	<div class="flex items-center justify-between gap-4">
		<h1 class="text-3xl font-bold text-gray-900">{data.spot.name}</h1>
		<button
			on:click={openModal}
			class="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-gray-300 text-sm text-gray-600 hover:border-gray-400 hover:text-gray-800 transition-colors cursor-pointer"
		>
			<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
					d="M3 21v-4m0 0V5a2 2 0 012-2h6.5l1 1H21l-3 6 3 6h-8.5l-1-1H5a2 2 0 00-2 2zm9-13.5V9" />
			</svg>
			Report conditions
		</button>
	</div>

	<!-- Combined weather + saturation chart -->
	{#if data.weather.length > 0}
		<div class="bg-white rounded-lg shadow-sm p-4">
			<WeatherChart weather={data.weather} conditions={data.conditions} />
		</div>
	{:else}
		<div class="bg-yellow-50 border border-yellow-200 rounded-lg p-8 text-center">
			<p class="text-yellow-800 font-medium">No weather data available yet</p>
			<p class="text-yellow-600 text-sm mt-1">Data is being fetched. Check back in a few minutes.</p>
		</div>
	{/if}

	<!-- Embed section -->
	<div class="border border-gray-200 rounded-lg overflow-hidden">
		<button
			on:click={() => showEmbed = !showEmbed}
			class="w-full px-4 py-3 flex items-center justify-between text-sm text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer"
		>
			<span class="font-medium">Embed on your website</span>
			<svg class="w-4 h-4 transition-transform {showEmbed ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
			</svg>
		</button>
		{#if showEmbed}
			<div class="border-t border-gray-200 p-4 space-y-3 bg-gray-50">
				<pre class="text-xs bg-white border border-gray-200 rounded p-3 overflow-x-auto text-gray-700 whitespace-pre-wrap">{iframeCode}</pre>
				<div class="flex items-center justify-between gap-3 flex-wrap">
					<button
						on:click={copyEmbed}
						class="px-3 py-1.5 text-sm bg-gray-900 text-white rounded-lg hover:bg-gray-700 cursor-pointer transition-colors"
					>
						{copied ? 'Copied!' : 'Copy code'}
					</button>
					<p class="text-xs text-gray-400">
						Using this on your site? <a href="mailto:dev@blocweather.com" class="text-blue-500 hover:underline">Let me know</a> to make my day!
					</p>
				</div>

			<!-- Condition report embed -->
			<div class="border-t border-gray-100 pt-3 space-y-2">
				<p class="text-xs font-medium text-gray-600">Condition report widget</p>
				<pre class="text-xs bg-white border border-gray-200 rounded p-3 overflow-x-auto text-gray-700 whitespace-pre-wrap">{iframeReportCode}</pre>
				<button
					on:click={copyEmbed2}
					class="px-3 py-1.5 text-sm bg-gray-900 text-white rounded-lg hover:bg-gray-700 cursor-pointer transition-colors"
				>
					{copied2 ? 'Copied!' : 'Copy code'}
				</button>
			</div>

			</div>
		{/if}
	</div>
</div>

<!-- Report modal -->
{#if showModal}
	<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
		on:click|self={() => { showModal = false; }}
	>
		<div class="bg-white rounded-xl shadow-xl w-full max-w-sm mx-4 p-6 space-y-5">
			<h2 class="text-lg font-semibold text-gray-900">Report conditions</h2>

			{#if submitSuccess}
				<div class="flex items-center gap-2 text-green-700 bg-green-50 border border-green-200 rounded-lg p-3">
					<svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
					</svg>
					<span class="text-sm font-medium">Report submitted. Thank you!</span>
				</div>
			{:else}
				<!-- Time picker -->
				<div class="space-y-1.5">
					<label for="observed-time" class="block text-sm font-medium text-gray-700">
						When did you observe this?
					</label>
					<input
						id="observed-time"
						type="datetime-local"
						step="3600"
						max={maxDatetime()}
						bind:value={observedTime}
						class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					/>
				</div>

				<!-- Status buttons -->
				<div class="space-y-1.5">
					<p class="text-sm font-medium text-gray-700">Rock conditions</p>
					<div class="grid grid-cols-2 gap-2">
						{#each Object.entries(STATUS_LABELS) as [status, label]}
							<button on:click={() => { selectedStatus = status; }} class="py-2 px-3 rounded-lg border text-sm font-medium transition-all cursor-pointer {selectedStatus === status ? STATUS_ACTIVE[status] : STATUS_STYLES[status]}">
								{label}
							</button>
						{/each}
					</div>
				</div>

				<!-- Comment -->
				<div class="space-y-1.5">
					<label for="report-comment" class="block text-sm font-medium text-gray-700">
						Comment <span class="text-gray-400 font-normal">(optional)</span>
					</label>
					<textarea
						id="report-comment"
						rows="2"
						maxlength="500"
						placeholder="e.g. north face still damp"
						bind:value={comment}
						class="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
					></textarea>
				</div>

				{#if submitError}
					<p class="text-sm text-red-600">{submitError}</p>
				{/if}

				<!-- Actions -->
				<div class="flex gap-2 pt-1">
					<button
						on:click={() => { showModal = false; }}
						class="flex-1 py-2 rounded-lg border border-gray-300 text-sm text-gray-600 hover:bg-gray-50 transition-colors cursor-pointer"
					>
						Cancel
					</button>
					<button
						on:click={submitReport}
						disabled={!selectedStatus || !observedTime || submitting}
						class="flex-1 py-2 rounded-lg bg-gray-900 text-sm text-white font-medium hover:bg-gray-700 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed"
					>
						{submitting ? 'Submitting…' : 'Submit'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
