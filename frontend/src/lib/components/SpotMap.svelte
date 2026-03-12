<script lang="ts">
	import { onMount, onDestroy, afterUpdate } from 'svelte';
	import { api } from '$lib/api/client';
	import type { WetnessClass } from '$lib/api/client';
	import 'leaflet/dist/leaflet.css';

	/** ISO timestamp string; undefined = now */
	export let timestamp: string | undefined = undefined;

	const CLASS_COLOR: Record<WetnessClass | 'unknown', string> = {
		dry:        '#16a34a',
		mostly_dry: '#65a30d',
		some_wet:   '#7dd3fc',
		mostly_wet: '#2563eb',
		wet:        '#1e3a8a',
		unknown:    '#9ca3af',
	};

	const CLASS_LABEL: Record<WetnessClass | 'unknown', string> = {
		dry:        'Dry',
		mostly_dry: 'Mostly dry',
		some_wet:   'Some wet',
		mostly_wet: 'Mostly wet',
		wet:        'Wet',
		unknown:    'No data',
	};

	let container: HTMLDivElement;
	let map: import('leaflet').Map | null = null;
	let layerGroup: import('leaflet').LayerGroup | null = null;
	let L: typeof import('leaflet') | null = null;

	async function loadSpots(swLat: number, swLon: number, neLat: number, neLon: number) {
		if (!L || !layerGroup) return;
		try {
			const spots = await api.getMapSpots({ swLat, swLon, neLat, neLon }, timestamp);
			layerGroup.clearLayers();
			for (const spot of spots) {
				const cls = (spot.classification ?? 'unknown') as WetnessClass | 'unknown';
				const color = CLASS_COLOR[cls];
				const label = CLASS_LABEL[cls];
				L.circleMarker([spot.latitude, spot.longitude], {
					radius: 7,
					fillColor: color,
					color: '#ffffff',
					weight: 1.5,
					opacity: 1,
					fillOpacity: 0.9,
				})
					.bindPopup(
						`<div style="text-align:center;min-width:120px">
							<strong>${spot.name}</strong><br>
							<span style="display:inline-block;margin:4px 0;padding:2px 8px;border-radius:9999px;background:${color};color:#fff;font-size:11px;font-weight:600">${label}</span><br>
							<a href="/${spot.country_slug}/${spot.region_slug}/${spot.spot_slug}" style="color:#2563eb;font-size:12px">View details →</a>
						</div>`
					)
					.addTo(layerGroup);
			}
		} catch {
			// silently ignore — map still usable
		}
	}

	onMount(async () => {
		L = (await import('leaflet')).default;

		map = L.map(container).setView([47, 10], 4);

		L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
			attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
			maxZoom: 18,
		}).addTo(map);

		layerGroup = L.layerGroup().addTo(map);

		await loadSpots(-90, -180, 90, 180);

		map.on('moveend', () => {
			const b = map!.getBounds();
			loadSpots(b.getSouth(), b.getWest(), b.getNorth(), b.getEast());
		});
	});

	// Reload when timestamp prop changes
	let prevTimestamp = timestamp;
	afterUpdate(() => {
		if (timestamp !== prevTimestamp) {
			prevTimestamp = timestamp;
			if (map) {
				const b = map.getBounds();
				loadSpots(b.getSouth(), b.getWest(), b.getNorth(), b.getEast());
			}
		}
	});

	onDestroy(() => {
		map?.remove();
	});
</script>

<div bind:this={container} class="h-full w-full"></div>
