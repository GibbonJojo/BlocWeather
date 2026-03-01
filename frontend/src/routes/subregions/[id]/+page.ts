import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const spots = await api.getSpotsBySubregion(params.id);
		return {
			subregionId: params.id,
			spots
		};
	} catch (err) {
		console.error('Failed to load spots:', err);
		throw error(404, 'Subregion not found');
	}
};
