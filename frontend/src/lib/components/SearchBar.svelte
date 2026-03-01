<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/client';
	import type { SearchResult, SearchResultKind } from '$lib/api/client';

	let query = '';
	let results: SearchResult[] = [];
	let open = false;
	let timer: ReturnType<typeof setTimeout>;
	let inputEl: HTMLInputElement;
	let activeIndex = -1;

	const HREF: Record<SearchResultKind, (id: string) => string> = {
		spot:      (id) => `/spots/${id}`,
		subregion: (id) => `/subregions/${id}`,
		country:   (id) => `/countries/${id}`,
	};

	const KIND_LABEL: Record<SearchResultKind, string> = {
		spot:      'Spot',
		subregion: 'Region',
		country:   'Country',
	};

	function onInput() {
		activeIndex = -1;
		clearTimeout(timer);
		if (query.trim().length < 2) {
			results = [];
			open = false;
			return;
		}
		timer = setTimeout(doSearch, 280);
	}

	async function doSearch() {
		try {
			results = await api.search(query);
			open = results.length > 0;
		} catch {
			results = [];
			open = false;
		}
	}

	function select(result: SearchResult) {
		query = '';
		results = [];
		open = false;
		goto(HREF[result.kind](result.id));
	}

	function onKeydown(e: KeyboardEvent) {
		if (!open) return;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			activeIndex = Math.min(activeIndex + 1, results.length - 1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			activeIndex = Math.max(activeIndex - 1, -1);
		} else if (e.key === 'Enter' && activeIndex >= 0) {
			e.preventDefault();
			select(results[activeIndex]);
		} else if (e.key === 'Escape') {
			open = false;
			query = '';
			inputEl?.blur();
		}
	}

	function onBlur() {
		// Delay so result clicks can fire first
		setTimeout(() => { open = false; }, 150);
	}

	function onFocus() {
		if (results.length > 0) open = true;
	}
</script>

<div class="relative w-full max-w-sm">
	<div class="relative">
		<svg
			class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400"
			fill="none" stroke="currentColor" viewBox="0 0 24 24"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
				d="M21 21l-4.35-4.35M17 11A6 6 0 1 1 5 11a6 6 0 0 1 12 0z" />
		</svg>
		<input
			bind:this={inputEl}
			bind:value={query}
			on:input={onInput}
			on:keydown={onKeydown}
			on:blur={onBlur}
			on:focus={onFocus}
			type="search"
			placeholder="Search spots, regions…"
			autocomplete="off"
			class="w-full rounded-lg border border-gray-200 bg-gray-50 pl-9 pr-3 py-1.5 text-sm text-gray-900 placeholder-gray-400
				focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:bg-white transition"
		/>
	</div>

	{#if open && results.length > 0}
		<ul
			class="absolute left-0 right-0 top-full mt-1 z-50 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden"
			role="listbox"
		>
			{#each results as result, i}
				<!-- svelte-ignore a11y-click-events-have-key-events -->
				<li
					role="option"
					aria-selected={activeIndex === i}
					on:click={() => select(result)}
					class="flex items-center justify-between px-3 py-2 cursor-pointer text-sm transition-colors
						{activeIndex === i ? 'bg-blue-50 text-blue-900' : 'text-gray-800 hover:bg-gray-50'}"
				>
					<span class="font-medium truncate">{result.name}</span>
					<div class="flex items-center gap-2 ml-2 shrink-0">
						{#if result.context}
							<span class="text-gray-400 text-xs truncate max-w-32">{result.context}</span>
						{/if}
						<span class="text-xs rounded px-1.5 py-0.5 font-medium
							{result.kind === 'spot' ? 'bg-blue-100 text-blue-700' :
							 result.kind === 'subregion' ? 'bg-purple-100 text-purple-700' :
							 'bg-green-100 text-green-700'}">
							{KIND_LABEL[result.kind]}
						</span>
					</div>
				</li>
			{/each}
		</ul>
	{/if}
</div>
