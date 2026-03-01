import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params, url }) => {
	try {
		const [spot, conditions, weather] = await Promise.all([
			api.getSpot(params.id),
			api.getSpotConditions(params.id),
			api.getSpotWeather(params.id),
		]);

		// ?show=temp,humidity,saturation,rain,rock,sun,wind
		const showParam = url.searchParams.get('show');
		const show = showParam ? showParam.split(',') : null;

		const initialActive = show ? {
			temperature:   show.includes('temp'),
			humidity:      show.includes('humidity'),
			precipitation: show.includes('rain'),
			saturation:    show.includes('saturation'),
			rockTemp:      show.includes('rock'),
			sunshine:      show.includes('sun'),
			wind:          show.includes('wind'),
		} : undefined;

		return { spot, conditions, weather, initialActive };
	} catch {
		throw error(404, 'Spot not found');
	}
};
