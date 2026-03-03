import { api } from '$lib/api/client';
import type { PageLoad } from './$types';
import { error } from '@sveltejs/kit';

export const load: PageLoad = async ({ params }) => {
	try {
		const spot = await api.getSpot(params.id);
		return { spot };
	} catch {
		throw error(404, 'Spot not found');
	}
};
