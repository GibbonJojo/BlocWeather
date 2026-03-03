export const ssr = false;

import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const spot = await api.getSpotBySlug(params.country_slug, params.region_slug, params.spot_slug);
		return { spot };
	} catch {
		throw error(404, 'Spot not found');
	}
};
