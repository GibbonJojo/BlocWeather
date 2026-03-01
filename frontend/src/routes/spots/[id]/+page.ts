import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		// Load spot details and current conditions in parallel
		const [spot, conditions, weather] = await Promise.all([
			api.getSpot(params.id),
			api.getSpotConditions(params.id),
			api.getSpotWeather(params.id)
		]);

		return {
			spot,
			conditions,
			weather
		};
	} catch (err) {
		console.error('Failed to load spot:', err);
		throw error(404, 'Spot not found');
	}
};
