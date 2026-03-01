// Formatting utilities for dates, temperatures, etc.
import { format } from 'date-fns';

export function formatDate(dateString: string): string {
	return format(new Date(dateString), 'MMM d, yyyy');
}

export function formatTime(dateString: string): string {
	return format(new Date(dateString), 'HH:mm');
}

export function formatDateTime(dateString: string): string {
	return format(new Date(dateString), 'MMM d, HH:mm');
}

export function formatTemperature(temp: number): string {
	return `${Math.round(temp)}°C`;
}

export function formatTemperatureRange(min: number, max: number): string {
	return `${Math.round(min)}-${Math.round(max)}°C`;
}

export function formatHumidity(humidity: number): string {
	return `${Math.round(humidity)}%`;
}

export function formatPrecipitation(precip: number): string {
	if (precip === 0) return '0 mm';
	if (precip < 0.1) return '< 0.1 mm';
	return `${precip.toFixed(1)} mm`;
}

export function formatWindSpeed(speed: number): string {
	return `${Math.round(speed)} km/h`;
}

export function formatWindDirection(degrees?: number): string {
	if (degrees === undefined || degrees === null) return 'N/A';

	const directions = ['N', 'NNE', 'NE', 'ENE', 'E', 'ESE', 'SE', 'SSE', 'S', 'SSW', 'SW', 'WSW', 'W', 'WNW', 'NW', 'NNW'];
	const index = Math.round(degrees / 22.5) % 16;
	return directions[index];
}

export function formatDryingTime(hours?: number): string {
	if (!hours) return 'Dry';

	if (hours < 1) {
		const minutes = Math.round(hours * 60);
		return `Dry in ~${minutes} min`;
	}

	return `Dry in ~${Math.round(hours)}h`;
}
