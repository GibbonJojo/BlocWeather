export const ssr = false;

import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const country = await api.getCountryBySlug(params.country_slug);
		return { country };
	} catch {
		throw error(404, 'Country not found');
	}
};
