import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const subregions = await api.getSubregions(params.id);
		return {
			countryId: params.id,
			subregions
		};
	} catch (err) {
		console.error('Failed to load subregions:', err);
		throw error(404, 'Country not found');
	}
};
