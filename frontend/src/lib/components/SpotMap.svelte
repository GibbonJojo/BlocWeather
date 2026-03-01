<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { api } from '$lib/api/client';
	import 'leaflet/dist/leaflet.css';

	let container: HTMLDivElement;
	let map: import('leaflet').Map | null = null;
	let layerGroup: import('leaflet').LayerGroup | null = null;

	async function loadSpots(L: typeof import('leaflet'), swLat: number, swLon: number, neLat: number, neLon: number) {
		try {
			const spots = await api.getMapSpots({ swLat, swLon, neLat, neLon });
			layerGroup!.clearLayers();
			for (const spot of spots) {
				L.circleMarker([spot.latitude, spot.longitude], {
					radius: 7,
					fillColor: '#2563eb',
					color: '#ffffff',
					weight: 1.5,
					opacity: 1,
					fillOpacity: 0.85,
				})
					.bindPopup(
						`<div style="text-align:center;min-width:110px">
							<strong>${spot.name}</strong><br>
							<a href="/spots/${spot.id}" style="color:#2563eb;font-size:12px">View details →</a>
						</div>`
					)
					.addTo(layerGroup!);
			}
		} catch {
			// silently ignore — map still usable
		}
	}

	onMount(async () => {
		const L = (await import('leaflet')).default;

		map = L.map(container).setView([47, 10], 4);

		L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
			attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
			maxZoom: 18,
		}).addTo(map);

		layerGroup = L.layerGroup().addTo(map);

		// Initial load — all spots worldwide
		await loadSpots(L, -90, -180, 90, 180);

		map.on('moveend', () => {
			const b = map!.getBounds();
			loadSpots(L, b.getSouth(), b.getWest(), b.getNorth(), b.getEast());
		});
	});

	onDestroy(() => {
		map?.remove();
	});
</script>

<div bind:this={container} class="h-full w-full"></div>
