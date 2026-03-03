# Submitting Condition Reports via API

You can submit rock condition reports directly via HTTP — useful if you want to build your own button, form, or integration instead of using the embedded widget.

## Endpoint

```
POST https://blocweather.com/api/v1/data/{country}/{region}/{spot}/reports
```

Where `{country}`, `{region}`, and `{spot}` are the slugs from the spot's URL on BlocWeather.

**Example:** for `blocweather.com/germany/nordrhein-westfalen/hohenfels` the endpoint is:
```
POST https://blocweather.com/api/v1/data/germany/nordrhein-westfalen/hohenfels/reports
```

For spots with no region, use `-` as the region: `/data/germany/-/hohenfels/reports`.

CORS is open, so this can be called from a browser as well as a server.

## Request

**Headers**
```
Content-Type: application/json
```

**Body**
```json
{
  "observed_at": "2025-06-15T14:00:00Z",
  "status": "dry"
}
```

| Field         | Type   | Description |
|---------------|--------|-------------|
| `observed_at` | string | ISO 8601 datetime (UTC) of when you observed the conditions. Must be in the past. |
| `status`      | string | One of: `dry`, `some_wet`, `mostly_wet`, `wet` |

**Status values**

| Value        | Meaning |
|--------------|---------|
| `dry`        | Rock is completely dry |
| `some_wet`   | A few wet patches, mostly climbable |
| `mostly_wet` | Largely wet, limited climbability |
| `wet`        | Rock is fully wet |

## Response

**201 Created** — report accepted.

```json
{
  "id": "...",
  "spot_id": "...",
  "observed_at": "2025-06-15T14:00:00Z",
  "status": "dry",
  "reported_at": "2025-06-15T14:32:00Z"
}
```

**400 Bad Request** — missing or invalid fields.
**404 Not Found** — spot ID does not exist.

## Example (curl)

```sh
curl -X POST https://blocweather.com/api/v1/data/germany/nordrhein-westfalen/hohenfels/reports \
  -H "Content-Type: application/json" \
  -d '{"observed_at": "2025-06-15T14:00:00Z", "status": "dry"}'
```

## Notes

- `observed_at` should reflect when you were actually at the crag, not when you submit the request. Reporting with the correct time improves the algorithm.
- Reports are used to validate and improve the rock wetness model — they are not displayed directly to users.
- There is no authentication required.
