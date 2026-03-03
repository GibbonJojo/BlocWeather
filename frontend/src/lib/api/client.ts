// API client for BlocWeather backend
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE = PUBLIC_API_URL || 'http://localhost:3000/api/v1';

export interface Country {
	id: string;
	name: string;
	code: string;
	slug: string;
	spot_count: number;
}

export interface Subregion {
	id: string;
	name: string;
	slug: string;
	country_id: string;
	spot_count: number;
}

export interface Spot {
	id: string;
	name: string;
	slug: string;
	latitude: number;
	longitude: number;
	country_id: string;
	subregion_id?: string;
	description?: string;
	elevation_meters?: number;
	rock_type?: string;
	exposure?: string;
	climbing_types?: string[];
	// Present when fetched via get_spot_handler or data_spot_handler
	country?: { id: string; name: string; code: string; slug: string };
	subregion?: { id: string; name: string; slug: string } | null;
	created_at?: string;
}

export interface SpotListItem {
	id: string;
	name: string;
	slug: string;
	latitude: number;
	longitude: number;
	rock_type: string;
	exposure: string;
	climbing_types: string[];
}

export interface CountryData {
	id: string;
	name: string;
	code: string;
	slug: string;
	subregions: Array<{ id: string; name: string; slug: string; spot_count: number }>;
}

export interface SubregionData {
	id: string;
	name: string;
	slug: string;
	country: { id: string; name: string; code: string; slug: string };
	spots: SpotListItem[];
}

export interface MapSpot {
	id: string;
	name: string;
	latitude: number;
	longitude: number;
	saturation?: number; // max_saturation 0–1 for map coloring
	country_slug: string;
	region_slug: string;
	spot_slug: string;
}

export interface WeatherData {
	timestamp: string;
	temperature_c: number;
	dewpoint_c: number | null;
	humidity_percent: number;
	precipitation_mm: number;
	cloud_cover_percent: number;
	wind_speed_kmh: number;
	wind_direction_degrees?: number;
	solar_radiation_wm2?: number;
	sunshine_duration_s: number | null;
	is_forecast: boolean;
}

export interface ClimbingCondition {
	timestamp: string;
	rock_surface_temp_min_c: number;
	rock_surface_temp_max_c: number;
	min_saturation: number; // 0–1, fast-drying (sunny) lower bound
	max_saturation: number; // 0–1, slow-drying (shaded) upper bound
}

export type SearchResultKind = 'spot' | 'subregion' | 'country';

export interface SearchResult {
	kind: SearchResultKind;
	id: string;
	name: string;
	context: string;
}

export type ConditionStatus = 'dry' | 'some_wet' | 'mostly_wet' | 'wet';

export interface ConditionReport {
	id: string;
	spot_id: string;
	observed_at: string;
	status: ConditionStatus;
	reported_at: string;
}

// ── Admin types ───────────────────────────────────────────────────────────────

export interface AdminSpot {
	id: string;
	name: string;
	slug: string;
	latitude: number;
	longitude: number;
	country_id: string;
	country_name: string;
	subregion_id?: string;
	subregion_name?: string;
	rock_type?: string;
	exposure?: string;
	elevation_meters?: number;
	description?: string;
	climbing_types: string[];
}

export interface AdminSubregion {
	id: string;
	name: string;
	country_id: string;
	country_name: string;
	spot_count: number;
}

export interface AreaSuggestion {
	id: string;
	name: string;
	country: string;
	created_at: string;
}

export interface AdminReport {
	id: string;
	spot_id: string;
	spot_name: string;
	observed_at: string;
	status: string;
	reported_at: string;
	calc_min_saturation?: number;
	calc_max_saturation?: number;
}

export interface CreateSpotData {
	name: string;
	latitude: number;
	longitude: number;
	country_id: string;
	subregion_id?: string;
	rock_type?: string;
	exposure?: string;
	elevation_meters?: number;
	description?: string;
	climbing_types: string[];
}

class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;
	}

	private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;

		try {
			const response = await fetch(url, {
				...options,
				headers: {
					'Content-Type': 'application/json',
					...options?.headers
				}
			});

			if (!response.ok) {
				throw new Error(`API error: ${response.status} ${response.statusText}`);
			}

			if (response.status === 204) {
				return undefined as unknown as T;
			}

			return response.json();
		} catch (error) {
			console.error(`API request failed: ${url}`, error);
			throw error;
		}
	}

	private auth(token: string): Record<string, string> {
		return { 'Authorization': `Bearer ${token}` };
	}

	// ── Public endpoints ──────────────────────────────────────────────────────

	async getCountries(): Promise<Country[]> {
		return this.request<Country[]>('/countries');
	}

	async getSubregions(countryId: string): Promise<Subregion[]> {
		return this.request<Subregion[]>(`/countries/${countryId}/subregions`);
	}

	async getSpotsBySubregion(subregionId: string): Promise<Spot[]> {
		return this.request<Spot[]>(`/subregions/${subregionId}/spots`);
	}

	async getSpot(spotId: string): Promise<Spot> {
		return this.request<Spot>(`/spots/${spotId}`);
	}

	async getSpotWeather(spotId: string, start?: string, end?: string): Promise<WeatherData[]> {
		const params = new URLSearchParams();
		if (start) params.append('start', start);
		if (end) params.append('end', end);
		const query = params.toString() ? `?${params.toString()}` : '';
		return this.request<WeatherData[]>(`/spots/${spotId}/weather${query}`);
	}

	async getSpotConditions(spotId: string, start?: string, end?: string): Promise<ClimbingCondition[]> {
		const params = new URLSearchParams();
		if (start) params.append('start', start);
		if (end) params.append('end', end);
		const query = params.toString() ? `?${params.toString()}` : '';
		return this.request<ClimbingCondition[]>(`/spots/${spotId}/conditions${query}`);
	}

	async getMapSpots(bounds: { swLat: number; swLon: number; neLat: number; neLon: number }): Promise<MapSpot[]> {
		const params = new URLSearchParams({
			sw_lat: bounds.swLat.toString(),
			sw_lon: bounds.swLon.toString(),
			ne_lat: bounds.neLat.toString(),
			ne_lon: bounds.neLon.toString(),
		});
		return this.request<MapSpot[]>(`/spots/map?${params.toString()}`);
	}

	async search(q: string): Promise<SearchResult[]> {
		if (q.trim().length < 2) return [];
		return this.request<SearchResult[]>(`/search?q=${encodeURIComponent(q.trim())}`);
	}

	async getCountryBySlug(countrySlug: string): Promise<CountryData> {
		return this.request<CountryData>(`/data/${countrySlug}`);
	}

	async getSubregionBySlug(countrySlug: string, regionSlug: string): Promise<SubregionData> {
		return this.request<SubregionData>(`/data/${countrySlug}/${regionSlug}`);
	}

	async getSpotBySlug(countrySlug: string, regionSlug: string, spotSlug: string): Promise<Spot> {
		return this.request<Spot>(`/data/${countrySlug}/${regionSlug}/${spotSlug}`);
	}

	async submitReport(spotId: string, observedAt: string, status: ConditionStatus): Promise<ConditionReport> {
		return this.request<ConditionReport>(`/spots/${spotId}/reports`, {
			method: 'POST',
			body: JSON.stringify({ observed_at: observedAt, status })
		});
	}

	// ── Admin endpoints ───────────────────────────────────────────────────────

	async adminLogin(username: string, password: string): Promise<{ token: string; username: string }> {
		return this.request('/admin/login', {
			method: 'POST',
			body: JSON.stringify({ username, password })
		});
	}

	// Spots
	async adminListSpots(token: string): Promise<AdminSpot[]> {
		return this.request('/admin/spots', { headers: this.auth(token) });
	}

	async adminCreateSpot(token: string, data: CreateSpotData): Promise<AdminSpot> {
		return this.request('/admin/spots', {
			method: 'POST',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminUpdateSpot(token: string, id: string, data: Partial<CreateSpotData>): Promise<AdminSpot> {
		return this.request(`/admin/spots/${id}`, {
			method: 'PUT',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminDeleteSpot(token: string, id: string): Promise<void> {
		return this.request(`/admin/spots/${id}`, {
			method: 'DELETE',
			headers: this.auth(token)
		});
	}

	// Countries
	async adminCreateCountry(token: string, data: { name: string; code: string }): Promise<{ id: string; name: string; code: string }> {
		return this.request('/admin/countries', {
			method: 'POST',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminUpdateCountry(token: string, id: string, data: { name?: string; code?: string }): Promise<{ id: string; name: string; code: string }> {
		return this.request(`/admin/countries/${id}`, {
			method: 'PUT',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminDeleteCountry(token: string, id: string): Promise<void> {
		return this.request(`/admin/countries/${id}`, {
			method: 'DELETE',
			headers: this.auth(token)
		});
	}

	// Subregions
	async adminListSubregions(token: string): Promise<AdminSubregion[]> {
		return this.request('/admin/subregions', { headers: this.auth(token) });
	}

	async adminCreateSubregion(token: string, data: { name: string; country_id: string }): Promise<{ id: string; name: string; country_id: string }> {
		return this.request('/admin/subregions', {
			method: 'POST',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminUpdateSubregion(token: string, id: string, data: { name: string }): Promise<{ id: string; name: string; country_id: string }> {
		return this.request(`/admin/subregions/${id}`, {
			method: 'PUT',
			headers: this.auth(token),
			body: JSON.stringify(data)
		});
	}

	async adminDeleteSubregion(token: string, id: string): Promise<void> {
		return this.request(`/admin/subregions/${id}`, {
			method: 'DELETE',
			headers: this.auth(token)
		});
	}

	// Reports
	async adminListReports(token: string): Promise<AdminReport[]> {
		return this.request('/admin/reports', { headers: this.auth(token) });
	}

	async adminDeleteReport(token: string, reportId: string): Promise<void> {
		return this.request(`/admin/reports/${reportId}`, {
			method: 'DELETE',
			headers: this.auth(token),
		});
	}

	// Area suggestions (public)
	async submitAreaSuggestion(name: string, country: string): Promise<void> {
		return this.request('/suggestions', {
			method: 'POST',
			body: JSON.stringify({ name, country })
		});
	}

	// Area suggestions (admin)
	async adminListSuggestions(token: string): Promise<AreaSuggestion[]> {
		return this.request('/admin/suggestions', { headers: this.auth(token) });
	}

	async adminDeleteSuggestion(token: string, id: string): Promise<void> {
		return this.request(`/admin/suggestions/${id}`, {
			method: 'DELETE',
			headers: this.auth(token)
		});
	}

	async adminDeleteAllSuggestions(token: string): Promise<void> {
		return this.request('/admin/suggestions', {
			method: 'DELETE',
			headers: this.auth(token)
		});
	}
}

export const api = new ApiClient(API_BASE);

export function rockWetnessLabel(): string {
	return Math.random() < 0.0001 ? 'Cock wetness' : 'Rock wetness';
}
