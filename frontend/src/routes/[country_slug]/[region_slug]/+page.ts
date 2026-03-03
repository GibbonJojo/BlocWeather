export const ssr = false;

import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const subregion = await api.getSubregionBySlug(params.country_slug, params.region_slug);
		return { subregion };
	} catch {
		throw error(404, 'Region not found');
	}
};
