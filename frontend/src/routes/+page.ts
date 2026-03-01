import { api } from '$lib/api/client';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	try {
		const countries = await api.getCountries();
		return {
			countries
		};
	} catch (error) {
		console.error('Failed to load countries:', error);
		return {
			countries: [],
			error: 'Failed to load countries. Please try again later.'
		};
	}
};
