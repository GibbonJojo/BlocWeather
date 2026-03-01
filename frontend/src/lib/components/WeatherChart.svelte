<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { WeatherData, ClimbingCondition } from '$lib/api/client';

	export let weather: WeatherData[];
	export let conditions: ClimbingCondition[];
	export let initialActive: Partial<Record<string, boolean>> | undefined = undefined;

	// ── Toggle state ──────────────────────────────────────────────────────────
	type Toggle = 'temperature' | 'humidity' | 'precipitation' | 'saturation' | 'rockTemp' | 'sunshine' | 'wind';

	let active: Record<string, boolean> = {
		temperature:   initialActive?.temperature   ?? true,
		humidity:      initialActive?.humidity      ?? true,
		precipitation: initialActive?.precipitation ?? true,
		saturation:    initialActive?.saturation    ?? true,
		rockTemp:      initialActive?.rockTemp      ?? false,
		sunshine:      initialActive?.sunshine      ?? false,
		wind:          initialActive?.wind          ?? false,
	};

	// Colors used for both toggle chips and chart series
	const COLORS: Record<string, string> = {
		temperature:   'rgb(239, 68, 68)',
		humidity:      'rgb(59, 130, 246)',
		precipitation: 'rgb(96, 165, 250)',
		saturation:    'rgb(99, 102, 241)',
		rockTemp:      'rgb(249, 115, 22)',
		sunshine:      'rgb(234, 179, 8)',
		wind:          'rgb(20, 184, 166)',
	};

	const TOGGLE_LABELS: Record<Toggle, string> = {
		temperature:   'Air Temp',
		humidity:      'Humidity',
		precipitation: 'Rain',
		saturation:    'Saturation',
		rockTemp:      'Rock Temp',
		sunshine:      'Sunshine',
		wind:          'Wind',
	};

	// Dataset indices — must match the datasets array order below
	const IDX = {
		temperature:   0,
		rockTempMin:   1,  // internal: bottom of rock temp band
		rockTempMax:   2,  // top of rock temp band, fill toward -1
		humidity:      3,
		satMin:        4,  // internal: bottom of saturation band
		satMax:        5,  // top of saturation band, fill toward -1
		precipitation: 6,
		sunshine:      7,
		wind:          8,
	};

	// ── Chart refs ────────────────────────────────────────────────────────────
	let canvas: HTMLCanvasElement;
	let chartInstance: any = null;

	// ── Toggle handler ────────────────────────────────────────────────────────
	function toggle(key: string) {
		active[key] = !active[key];
		active = active; // trigger Svelte reactivity for chip styling
		if (!chartInstance) return;

		const ds = chartInstance.data.datasets;
		switch (key) {
			case 'temperature':
				ds[IDX.temperature].hidden = !active.temperature;
				break;
			case 'rockTemp':
				ds[IDX.rockTempMin].hidden = !active.rockTemp;
				ds[IDX.rockTempMax].hidden = !active.rockTemp;
				break;
			case 'humidity':
				ds[IDX.humidity].hidden = !active.humidity;
				break;
			case 'saturation':
				ds[IDX.satMin].hidden = !active.saturation;
				ds[IDX.satMax].hidden = !active.saturation;
				break;
			case 'precipitation':
				ds[IDX.precipitation].hidden = !active.precipitation;
				break;
			case 'sunshine':
				ds[IDX.sunshine].hidden = !active.sunshine;
				break;
			case 'wind':
				ds[IDX.wind].hidden = !active.wind;
				chartInstance.options.scales.yWind.display = active.wind;
				break;
		}
		chartInstance.update();
	}

	// ── Chart init ────────────────────────────────────────────────────────────
	onMount(async () => {
		if (!weather.length) return;

		const { Chart } = await import('chart.js/auto');

		// Conditions lookup by timestamp string
		const condMap = new Map(conditions.map(c => [c.timestamp, c]));

		// Data series
		const timestamps  = weather.map(w => w.timestamp);
		const temps       = weather.map(w => w.temperature_c);
		const humidity    = weather.map(w => w.humidity_percent);
		const precip      = weather.map(w => w.precipitation_mm);
		// Sunshine: seconds per hour → % of full sun (3600s = 100%)
		const sunshine    = weather.map(w =>
			w.sunshine_duration_s != null ? (w.sunshine_duration_s / 3600) * 100 : null
		);
		const wind        = weather.map(w => w.wind_speed_kmh);
		const rockTempMin = weather.map(w => condMap.get(w.timestamp)?.rock_surface_temp_min_c ?? null);
		const rockTempMax = weather.map(w => condMap.get(w.timestamp)?.rock_surface_temp_max_c ?? null);
		const satMin      = weather.map(w => {
			const c = condMap.get(w.timestamp);
			return c != null ? c.min_saturation * 100 : null;
		});
		const satMax      = weather.map(w => {
			const c = condMap.get(w.timestamp);
			return c != null ? c.max_saturation * 100 : null;
		});

		// Tick positions at midnight (for day-separator lines) and noon (for labels)
		const midnightIndices = weather
			.map((w, i) => ({ i, h: new Date(w.timestamp).getHours() }))
			.filter(({ h }) => h === 0)
			.map(({ i }) => i);
		const noonIndices = weather
			.map((w, i) => ({ i, h: new Date(w.timestamp).getHours() }))
			.filter(({ h }) => h === 12)
			.map(({ i }) => i);

		// Index of data point closest to current time (for "now" line)
		const now = Date.now();
		let nowIndex = 0, minDiff = Infinity;
		weather.forEach((w, i) => {
			const diff = Math.abs(new Date(w.timestamp).getTime() - now);
			if (diff < minDiff) { minDiff = diff; nowIndex = i; }
		});

		// Precipitation axis: bars should only fill the bottom ~20% of chart
		const maxPrecip = Math.max(...precip, 0.5);
		const precipAxisMax = Math.max(maxPrecip * 5, 5);

		// ── Midnight vertical lines plugin ────────────────────────────────
		const midnightLinesPlugin = {
			id: 'midnightLines',
			afterDraw(chart: any) {
				const { ctx, chartArea: { top, bottom }, scales: { x } } = chart;
				if (!x) return;
				ctx.save();
				ctx.lineWidth = 1;
				ctx.strokeStyle = 'rgba(0, 0, 0, 0.12)';
				for (const idx of midnightIndices) {
					const xPos = x.getPixelForValue(idx);
					ctx.beginPath();
					ctx.moveTo(xPos, top);
					ctx.lineTo(xPos, bottom);
					ctx.stroke();
				}
				ctx.restore();
			}
		};

		// ── "Now" vertical dotted line plugin ─────────────────────────────
		const nowLinePlugin = {
			id: 'nowLine',
			afterDraw(chart: any) {
				const { ctx, chartArea: { top, bottom }, scales: { x } } = chart;
				if (!x) return;
				const xPos = x.getPixelForValue(nowIndex);
				ctx.save();
				ctx.beginPath();
				ctx.moveTo(xPos, top);
				ctx.lineTo(xPos, bottom);
				ctx.lineWidth = 2;
				ctx.strokeStyle = 'rgba(30, 30, 30, 0.65)';
				ctx.setLineDash([5, 4]);
				ctx.stroke();
				ctx.setLineDash([]);
				ctx.fillStyle = 'rgba(30, 30, 30, 0.75)';
				ctx.font = 'bold 11px system-ui, sans-serif';
				ctx.textAlign = 'center';
				ctx.fillText('now', xPos, top - 6);
				ctx.restore();
			}
		};

		chartInstance = new Chart(canvas, {
			data: {
				// Numeric indices as x-values; dates rendered via tick callback
				labels: weather.map((_, i) => i),
				datasets: [
					// 0 ─ Air temperature
					{
						type: 'line',
						label: 'Air Temp (°C)',
						data: temps,
						borderColor: COLORS.temperature,
						backgroundColor: 'transparent',
						yAxisID: 'yTemp',
						tension: 0.3,
						pointRadius: 0,
						borderWidth: 2,
					},
					// 1 ─ Rock temp min — bottom of band (internal, no border/fill)
					{
						type: 'line',
						label: '_rockTempMin',
						data: rockTempMin,
						borderColor: 'transparent',
						backgroundColor: 'transparent',
						yAxisID: 'yTemp',
						tension: 0.3,
						pointRadius: 0,
						hidden: true,
						fill: false,
					},
					// 2 ─ Rock temp max — top of band, fills down to dataset -1
					{
						type: 'line',
						label: 'Rock Temp (°C)',
						data: rockTempMax,
						borderColor: COLORS.rockTemp,
						backgroundColor: 'rgba(249, 115, 22, 0.18)',
						yAxisID: 'yTemp',
						tension: 0.3,
						pointRadius: 0,
						hidden: true,
						fill: '-1',
					},
					// 3 ─ Humidity
					{
						type: 'line',
						label: 'Humidity (%)',
						data: humidity,
						borderColor: COLORS.humidity,
						backgroundColor: 'transparent',
						yAxisID: 'yPercent',
						tension: 0.3,
						pointRadius: 0,
						borderWidth: 2,
					},
					// 4 ─ Saturation min — bottom of band (internal)
					{
						type: 'line',
						label: '_satMin',
						data: satMin,
						borderColor: 'transparent',
						backgroundColor: 'transparent',
						yAxisID: 'yPercent',
						tension: 0.3,
						pointRadius: 0,
						fill: false,
					},
					// 5 ─ Saturation max — top of band, fills down to dataset -1
					{
						type: 'line',
						label: 'Saturation (%)',
						data: satMax,
						borderColor: COLORS.saturation,
						backgroundColor: 'rgba(99, 102, 241, 0.22)',
						yAxisID: 'yPercent',
						tension: 0.3,
						pointRadius: 0,
						fill: '-1',
					},
					// 6 ─ Precipitation bars
					{
						type: 'bar',
						label: 'Rain (mm)',
						data: precip,
						backgroundColor: 'rgba(96, 165, 250, 0.55)',
						yAxisID: 'yPrecip',
						borderWidth: 0,
						barPercentage: 1.0,
						categoryPercentage: 1.0,
					},
					// 7 ─ Sunshine (normalised to 0–100%)
					{
						type: 'line',
						label: 'Sunshine (%)',
						data: sunshine,
						borderColor: COLORS.sunshine,
						backgroundColor: 'rgba(234, 179, 8, 0.12)',
						yAxisID: 'yPercent',
						tension: 0.3,
						pointRadius: 0,
						hidden: true,
						fill: 'origin',
					},
					// 8 ─ Wind speed
					{
						type: 'line',
						label: 'Wind (km/h)',
						data: wind,
						borderColor: COLORS.wind,
						backgroundColor: 'transparent',
						yAxisID: 'yWind',
						tension: 0.3,
						pointRadius: 0,
						hidden: true,
						borderDash: [4, 4],
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				interaction: { mode: 'index', intersect: false },
				plugins: {
					legend: { display: false },
					tooltip: {
						callbacks: {
							title: (items: any[]) => {
								const i = items[0].dataIndex;
								const d = new Date(timestamps[i]);
								return d.toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' })
									+ ' · ' + d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
							},
							label: (ctx: any) => {
								if (ctx.dataset.label.startsWith('_')) return null;
								const v = ctx.parsed.y;
								if (v === null || v === undefined) return null;
								const i = ctx.dataIndex;
								const ds = ctx.chart.data.datasets;
								if (ctx.dataset.label === 'Saturation (%)') {
									const minV = ds[IDX.satMin].data[i];
									if (minV != null) return ` Saturation: ${(+minV).toFixed(0)}% – ${v.toFixed(0)}%`;
								}
								if (ctx.dataset.label === 'Rock Temp (°C)') {
									const minV = ds[IDX.rockTempMin].data[i];
									if (minV != null) return ` Rock Temp: ${(+minV).toFixed(1)}°C – ${v.toFixed(1)}°C`;
								}
								return ` ${ctx.dataset.label}: ${v.toFixed(1)}`;
							},
						},
					},
				},
				scales: {
					x: {
						type: 'linear',
						display: true,
						min: 0,
						max: weather.length - 1,
						offset: false,
						afterBuildTicks: (axis: any) => {
							axis.ticks = noonIndices.map((v: number) => ({ value: v }));
						},
						ticks: {
							maxRotation: 30,
							minRotation: 30,
							callback: (val: any) => {
								const i = Math.round(val);
								if (i < 0 || i >= weather.length) return '';
								const d = new Date(weather[i].timestamp);
								return d.toLocaleDateString(undefined, { weekday: 'short', day: 'numeric', month: 'short' });
							},
						},
						grid: { drawOnChartArea: false },
					},
					yTemp: {
						type: 'linear',
						position: 'left',
						title: { display: true, text: '°C', font: { size: 11 } },
						grid: { color: 'rgba(0, 0, 0, 0.06)' },
					},
					yPercent: {
						type: 'linear',
						position: 'right',
						min: 0,
						max: 100,
						title: { display: true, text: '%', font: { size: 11 } },
						grid: { drawOnChartArea: false },
					},
					// Precipitation uses a hidden axis; max is inflated so bars stay visually short
					yPrecip: {
						type: 'linear',
						position: 'right',
						display: false,
						min: 0,
						max: precipAxisMax,
						grid: { drawOnChartArea: false },
					},
					// Wind axis — only shown when wind toggle is active
					yWind: {
						type: 'linear',
						position: 'right',
						display: false,
						min: 0,
						title: { display: true, text: 'km/h', font: { size: 11 } },
						grid: { drawOnChartArea: false },
					},
				},
			},
			plugins: [midnightLinesPlugin, nowLinePlugin],
		});
	});

	onDestroy(() => {
		chartInstance?.destroy();
	});

	// Min chart pixel width so mobile can horizontally scroll
	$: minChartWidth = Math.max(weather.length * 5, 600);

</script>

<!-- Toggle chips -->
<div class="flex flex-wrap gap-2 mb-3">
	{#each Object.entries(TOGGLE_LABELS) as [key, label]}
		<button
			on:click={() => toggle(key)}
			class="px-3 py-1 rounded-full text-sm font-medium transition-all border cursor-pointer"
			style={active[key]
				? `background-color: ${COLORS[key]}22; border-color: ${COLORS[key]}; color: ${COLORS[key]};`
				: 'background-color: transparent; border-color: #d1d5db; color: #9ca3af;'}
		>
			{label}
		</button>
	{/each}
</div>

<!-- Scrollable chart wrapper -->
<div class="overflow-x-auto rounded">
	<div style="min-width: {minChartWidth}px; height: 320px;">
		<canvas bind:this={canvas} style="width: 100%; height: 100%;"></canvas>
	</div>
</div>
