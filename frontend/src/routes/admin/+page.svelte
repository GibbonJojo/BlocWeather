<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import SpotsSection from './SpotsSection.svelte';
	import CountriesSection from './CountriesSection.svelte';
	import RegionsSection from './RegionsSection.svelte';
	import ReportsSection from './ReportsSection.svelte';
	import SuggestionsSection from './SuggestionsSection.svelte';

	type Tab = 'spots' | 'countries' | 'regions' | 'reports' | 'suggestions';

	let token = '';
	let username = '';
	let activeTab: Tab = 'spots';

	// Login form state
	let loginUser = '';
	let loginPass = '';
	let loginError = '';
	let loggingIn = false;

	onMount(() => {
		token    = localStorage.getItem('admin_token') ?? '';
		username = localStorage.getItem('admin_username') ?? '';
	});

	async function login() {
		if (!loginUser || !loginPass) { loginError = 'Enter username and password.'; return; }
		loggingIn = true; loginError = '';
		try {
			const res = await api.adminLogin(loginUser, loginPass);
			token    = res.token;
			username = res.username;
			localStorage.setItem('admin_token',    token);
			localStorage.setItem('admin_username', username);
			loginPass = '';
		} catch {
			loginError = 'Invalid credentials.';
		} finally {
			loggingIn = false;
		}
	}

	function logout() {
		token = ''; username = '';
		localStorage.removeItem('admin_token');
		localStorage.removeItem('admin_username');
	}

	const TABS: { id: Tab; label: string }[] = [
		{ id: 'spots',     label: 'Spots' },
		{ id: 'countries', label: 'Countries' },
		{ id: 'regions',   label: 'Regions' },
		{ id: 'reports',     label: 'Reports' },
		{ id: 'suggestions', label: 'Suggestions' },
	];
</script>

<svelte:head>
	<title>Admin — BlocWeather</title>
</svelte:head>

{#if !token}
	<!-- Login -->
	<div class="max-w-sm mx-auto mt-16 space-y-6">
		<h1 class="text-2xl font-bold text-gray-900">Admin login</h1>
		<div class="space-y-3">
			<input
				bind:value={loginUser}
				type="text"
				placeholder="Username"
				autocomplete="username"
				class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
			/>
			<input
				bind:value={loginPass}
				type="password"
				placeholder="Password"
				autocomplete="current-password"
				on:keydown={(e) => e.key === 'Enter' && login()}
				class="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
			/>
			{#if loginError}<p class="text-red-600 text-sm">{loginError}</p>{/if}
			<button
				on:click={login}
				disabled={loggingIn}
				class="w-full py-2 bg-gray-900 text-white text-sm font-medium rounded-lg hover:bg-gray-700 disabled:opacity-40 cursor-pointer"
			>
				{loggingIn ? 'Logging in…' : 'Log in'}
			</button>
		</div>
	</div>

{:else}
	<!-- Dashboard -->
	<div class="space-y-6">
		<div class="flex items-center justify-between">
			<h1 class="text-2xl font-bold text-gray-900">Admin</h1>
			<div class="flex items-center gap-3 text-sm text-gray-500">
				<span>{username}</span>
				<button on:click={logout} class="text-red-500 hover:underline cursor-pointer">Log out</button>
			</div>
		</div>

		<!-- Tabs -->
		<div class="border-b border-gray-200">
			<nav class="flex gap-1">
				{#each TABS as tab}
					<button
						on:click={() => activeTab = tab.id}
						class="px-4 py-2 text-sm font-medium transition-colors cursor-pointer
							{activeTab === tab.id
								? 'border-b-2 border-gray-900 text-gray-900'
								: 'text-gray-500 hover:text-gray-700'}"
					>
						{tab.label}
					</button>
				{/each}
			</nav>
		</div>

		<!-- Tab content -->
		<div>
			{#if activeTab === 'spots'}
				<SpotsSection {token} />
			{:else if activeTab === 'countries'}
				<CountriesSection {token} />
			{:else if activeTab === 'regions'}
				<RegionsSection {token} />
			{:else if activeTab === 'reports'}
				<ReportsSection {token} />
			{:else if activeTab === 'suggestions'}
				<SuggestionsSection {token} />
			{/if}
		</div>
	</div>
{/if}
