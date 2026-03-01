// Color utilities for dry rock indicators and UI

export type DryRockColor = 'green' | 'orange' | 'red';

export function getDryRockColorHex(color: DryRockColor): string {
	switch (color) {
		case 'green':
			return '#22c55e'; // Green-500
		case 'orange':
			return '#f97316'; // Orange-500
		case 'red':
			return '#ef4444'; // Red-500
		default:
			return '#6b7280'; // Gray-500
	}
}

export function getDryRockColorClass(color: DryRockColor): string {
	switch (color) {
		case 'green':
			return 'bg-green-500 text-white';
		case 'orange':
			return 'bg-orange-500 text-white';
		case 'red':
			return 'bg-red-500 text-white';
		default:
			return 'bg-gray-500 text-white';
	}
}

export function getDryRockLabel(color: DryRockColor): string {
	switch (color) {
		case 'green':
			return 'Dry';
		case 'orange':
			return 'Damp';
		case 'red':
			return 'Wet';
		default:
			return 'Unknown';
	}
}

export function getFrictionQualityColor(quality: string): string {
	switch (quality) {
		case 'excellent':
			return '#22c55e'; // Green
		case 'good':
			return '#84cc16'; // Lime
		case 'fair':
			return '#f59e0b'; // Amber
		case 'poor':
			return '#ef4444'; // Red
		default:
			return '#6b7280'; // Gray
	}
}
