export const ssr = false;

import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const spot = await api.getSpotBySlug(params.country_slug, params.region_slug, params.spot_slug);
		const forecastStart = new Date(); forecastStart.setMinutes(0, 0, 0);
		const forecastEnd = new Date(forecastStart.getTime() + 14 * 24 * 3600 * 1000);
		const [conditions, weather, forecast14Weather, forecast14Conditions] = await Promise.all([
			api.getSpotConditions(spot.id),
			api.getSpotWeather(spot.id),
			api.getSpotWeather(spot.id, forecastStart.toISOString(), forecastEnd.toISOString()),
			api.getSpotConditions(spot.id, forecastStart.toISOString(), forecastEnd.toISOString()),
		]);
		return { spot, conditions, weather, forecast14Weather, forecast14Conditions };
	} catch {
		throw error(404, 'Spot not found');
	}
};
