# BlocWeather validation report generator
# Requirements: pip install requests matplotlib
#
# Run from project root:
#   python backend/validation_data/generate_report.py
#
# Output: backend/validation_data/validation_report.html

import json
import requests
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime, date, timedelta
from io import BytesIO
import base64
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# ---------------------------------------------------------------------------
# Load data
# ---------------------------------------------------------------------------

with open(os.path.join(SCRIPT_DIR, 'dryness.json')) as f:
    observations = json.load(f)

with open(os.path.join(SCRIPT_DIR, 'locations.json')) as f:
    locations = {loc['name']: loc for loc in json.load(f)}

DRYNESS_LABELS = {
    0: ('Dry',           '#22c55e'),
    1: ('Partially dry', '#eab308'),
    2: ('Mostly wet',    '#f97316'),
    3: ('Wet',           '#ef4444'),
}

# ---------------------------------------------------------------------------
# Open-Meteo fetch
# ---------------------------------------------------------------------------

def fetch_weather(lat, lon, start_date: date, end_date: date) -> dict:
    today = date.today()
    # Archive endpoint covers up to ~5 days ago; use forecast for recent dates
    if end_date <= today - timedelta(days=6):
        url = 'https://archive-api.open-meteo.com/v1/archive'
    else:
        url = 'https://api.open-meteo.com/v1/forecast'

    params = {
        'latitude':    lat,
        'longitude':   lon,
        'start_date':  start_date.isoformat(),
        'end_date':    end_date.isoformat(),
        'hourly':      'temperature_2m,precipitation,relative_humidity_2m,dew_point_2m,wind_speed_10m,shortwave_radiation',
        'timezone':    'UTC',
        'wind_speed_unit': 'kmh',
    }
    resp = requests.get(url, params=params, timeout=30)
    resp.raise_for_status()
    return resp.json()

# ---------------------------------------------------------------------------
# Physics models (mirrors Rust implementation exactly)
# ---------------------------------------------------------------------------

def rock_surface_temp(air_temp: float, solar_wm2: float):
    min_temp   = air_temp - 2.0
    solar_gain = max(0.0, (solar_wm2 - 150.0) / 900.0 * 15.0)
    max_temp   = air_temp + solar_gain
    return min_temp, max_temp

def saturation_step(precip, temp, dewpoint, humidity, wind, rock_min, rock_max, prev_min, prev_max):
    BASE_RATE = 3.5
    CAP       = 5.0

    if precip > 0.7:
        return min(prev_min + precip / CAP, 1.0), min(prev_max + precip / CAP, 1.0)

    eff = 0.0 if precip < 0.15 else precip
    x = eff / 0.7
    rate_reducing = 1.0 - x * x * (3.0 - 2.0 * x)

    hum_factor = (1.0 - humidity / 100.0) ** 0.5

    # Fast-drying: exposed rock
    spread_fast = rock_max - dewpoint
    if   spread_fast < 0.0: low = 0.0
    elif spread_fast < 2.0: low = BASE_RATE * spread_fast / 2.0
    else:                   low = BASE_RATE
    dr_fast = low * max(0.1, min(1.0, rock_max / 15.0)) * max(0.1, min(1.0, wind / 40.0)) * hum_factor
    new_min  = max(0.0, min(1.0, prev_min + eff / CAP - dr_fast * rate_reducing / CAP))

    # Slow-drying: shaded rock, half wind
    spread_slow = rock_min - dewpoint
    if   spread_slow < 0.0: high = 0.0
    elif spread_slow < 2.0: high = BASE_RATE * spread_slow / 2.0
    else:                   high = BASE_RATE
    dr_slow = high * max(0.1, min(1.0, rock_min / 15.0)) * (max(0.1, min(1.0, wind / 40.0)) * 0.5) * hum_factor
    new_max  = max(0.0, min(1.0, prev_max + eff / CAP - dr_slow * rate_reducing / CAP))

    return new_min, new_max

# ---------------------------------------------------------------------------
# Chart generation
# ---------------------------------------------------------------------------

def make_chart(obs: dict, loc: dict) -> str:
    obs_date   = date.fromisoformat(obs['date'])
    start_date = obs_date - timedelta(days=10)
    end_date   = obs_date + timedelta(days=2)

    data = fetch_weather(loc['latitude'], loc['longitude'], start_date, end_date)
    h    = data['hourly']

    times    = [datetime.fromisoformat(t) for t in h['time']]
    temps    = h['temperature_2m']
    precip   = h['precipitation']
    humidity = h['relative_humidity_2m']
    dewpoint = h['dew_point_2m']
    wind     = h['wind_speed_10m']
    solar    = h['shortwave_radiation']

    # Rock temperature per hour
    rock_mins = []; rock_maxs = []
    for t, s in zip(temps, solar):
        rm, rx = rock_surface_temp(t, s)
        rock_mins.append(rm); rock_maxs.append(rx)

    # Cumulative saturation — initialised at 100 %
    sat_min = []; sat_max = []
    prev_min = prev_max = 1.0
    for i in range(len(times)):
        nm, nx = saturation_step(
            precip[i], temps[i], dewpoint[i], humidity[i], wind[i],
            rock_mins[i], rock_maxs[i], prev_min, prev_max
        )
        sat_min.append(nm * 100); sat_max.append(nx * 100)
        prev_min, prev_max = nm, nx

    obs_dt = datetime(obs_date.year, obs_date.month, obs_date.day, 12)
    d_label, d_color = DRYNESS_LABELS.get(obs.get('dryness', -1), ('Unknown', '#9ca3af'))

    # --- Plot ---
    fig, axes = plt.subplots(3, 1, figsize=(14, 9), sharex=True,
                             gridspec_kw={'height_ratios': [2.5, 1.8, 1.2]})
    fig.patch.set_facecolor('#f8fafc')
    for ax in axes:
        ax.set_facecolor('#f8fafc')
        ax.grid(axis='y', color='#e2e8f0', linewidth=0.8)
        ax.spines[['top', 'right']].set_visible(False)
        # midnight lines
        for d in range(-10, 3):
            midnight = datetime(obs_date.year, obs_date.month, obs_date.day) + timedelta(days=d - (obs_date.day - obs_date.day))
        cur = start_date
        while cur <= end_date + timedelta(days=1):
            dt = datetime(cur.year, cur.month, cur.day)
            ax.axvline(dt, color='black', linewidth=0.8, alpha=0.15, zorder=1)
            cur += timedelta(days=1)

    AX_WET, AX_TEMP, AX_PRECIP = axes

    # Panel 1 — Rock Wetness
    AX_WET.fill_between(times, sat_min, sat_max, alpha=0.25, color='#6366f1')
    AX_WET.plot(times, sat_min, color='#6366f1', linewidth=1.8, label='Min wetness (exposed)')
    AX_WET.plot(times, sat_max, color='#6366f1', linewidth=1.8, linestyle='--', alpha=0.6, label='Max wetness (shaded)')
    AX_WET.axvline(obs_dt, color=d_color, linewidth=2.5, zorder=5, label=f'Visit: {d_label}')
    AX_WET.set_ylabel('Rock Wetness (%)', fontsize=9)
    AX_WET.set_ylim(0, 108)
    AX_WET.yaxis.set_major_formatter(plt.FuncFormatter(lambda v, _: f'{v:.0f}%'))
    AX_WET.legend(loc='upper right', fontsize=8, framealpha=0.9)

    # Panel 2 — Temperature
    AX_TEMP.fill_between(times, rock_mins, rock_maxs, alpha=0.12, color='#f97316', label='Rock temp range')
    AX_TEMP.plot(times, temps,    color='#374151', linewidth=1.5, label='Air temp')
    AX_TEMP.plot(times, dewpoint, color='#3b82f6', linewidth=1.2, linestyle=':', alpha=0.9, label='Dewpoint')
    AX_TEMP.axhline(0, color='#94a3b8', linewidth=0.8, linestyle='--', alpha=0.6)
    AX_TEMP.axvline(obs_dt, color=d_color, linewidth=2.5, zorder=5)
    AX_TEMP.set_ylabel('Temperature (°C)', fontsize=9)
    AX_TEMP.legend(loc='upper right', fontsize=8, framealpha=0.9)

    # Panel 3 — Precipitation
    AX_PRECIP.bar(times, precip, width=1/24, color='#3b82f6', alpha=0.75)
    AX_PRECIP.axvline(obs_dt, color=d_color, linewidth=2.5, zorder=5)
    AX_PRECIP.set_ylabel('Precip (mm/h)', fontsize=9)

    # X-axis
    AX_PRECIP.xaxis.set_major_formatter(mdates.DateFormatter('%b %d'))
    AX_PRECIP.xaxis.set_major_locator(mdates.DayLocator())
    plt.setp(AX_PRECIP.xaxis.get_majorticklabels(), rotation=30, ha='right', fontsize=8)

    title = f"{obs['location'].replace('_', ' ').title()}  ·  {obs['date']}"
    if 'comment' in obs:
        title += f"\n{obs['comment']}"
    fig.suptitle(title, fontsize=12, fontweight='bold', y=0.99)
    plt.tight_layout(rect=[0, 0, 1, 0.97])

    buf = BytesIO()
    plt.savefig(buf, format='png', dpi=130, bbox_inches='tight', facecolor=fig.get_facecolor())
    plt.close(fig)
    buf.seek(0)
    return base64.b64encode(buf.read()).decode()

# ---------------------------------------------------------------------------
# HTML report
# ---------------------------------------------------------------------------

BADGE_CSS = {0: 'dry', 1: 'partial', 2: 'mostly-wet', 3: 'wet'}

print("Generating validation report...")
cards_html = []
for obs in observations:
    loc_name = obs['location']
    if loc_name not in locations:
        print(f"  WARNING: no coordinates for '{loc_name}', skipping")
        continue
    print(f"  {loc_name}  {obs['date']}  (dryness={obs.get('dryness')})")
    try:
        img = make_chart(obs, locations[loc_name])
        d = obs.get('dryness', -1)
        d_label = DRYNESS_LABELS.get(d, ('Unknown', ''))[0]
        badge_class = BADGE_CSS.get(d, 'unknown')
        comment = f'<p class="comment">💬 {obs["comment"]}</p>' if 'comment' in obs else ''
        cards_html.append(f'''
<div class="card">
  <div class="card-header">
    <div>
      <span class="loc">{obs["location"].replace("_", " ").title()}</span>
      <span class="date">{obs["date"]}</span>
    </div>
    <span class="badge {badge_class}">{d} — {d_label}</span>
  </div>
  {comment}
  <img src="data:image/png;base64,{img}" />
</div>''')
    except Exception as e:
        print(f"  ERROR: {e}")
        cards_html.append(f'<div class="card error"><p>Failed for {loc_name} / {obs["date"]}: {e}</p></div>')

html = f'''<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>BlocWeather — Validation Report</title>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
         background: #f1f5f9; padding: 28px; color: #1e293b; }}
  h1 {{ font-size: 1.4rem; font-weight: 700; margin-bottom: 4px; }}
  .sub {{ color: #64748b; font-size: 0.85rem; margin-bottom: 20px; }}
  .note {{ background: #fffbeb; border: 1px solid #fcd34d; border-radius: 8px;
           padding: 10px 14px; font-size: 0.82rem; color: #92400e; margin-bottom: 28px; }}
  .card {{ background: white; border-radius: 12px; box-shadow: 0 1px 6px rgba(0,0,0,0.08);
           margin-bottom: 28px; overflow: hidden; }}
  .card-header {{ padding: 14px 18px; border-bottom: 1px solid #e2e8f0;
                  display: flex; align-items: center; justify-content: space-between; gap: 12px; }}
  .loc  {{ font-size: 1rem; font-weight: 600; }}
  .date {{ font-size: 0.82rem; color: #64748b; margin-left: 10px; }}
  .comment {{ padding: 8px 18px; font-size: 0.83rem; color: #64748b; font-style: italic;
              border-bottom: 1px solid #f1f5f9; background: #f8fafc; }}
  .badge {{ padding: 3px 11px; border-radius: 9999px; font-size: 0.78rem; font-weight: 600; white-space: nowrap; }}
  .badge.wet        {{ background: #fee2e2; color: #b91c1c; }}
  .badge.mostly-wet {{ background: #ffedd5; color: #c2410c; }}
  .badge.partial    {{ background: #fef9c3; color: #a16207; }}
  .badge.dry        {{ background: #dcfce7; color: #15803d; }}
  .badge.unknown    {{ background: #f1f5f9; color: #64748b; }}
  .card img {{ display: block; width: 100%; }}
  .card.error {{ padding: 16px 18px; color: #b91c1c; font-size: 0.85rem; }}
</style>
</head>
<body>
<h1>BlocWeather — Validation Report</h1>
<p class="sub">Generated {datetime.utcnow().strftime("%Y-%m-%d %H:%M")} UTC &nbsp;·&nbsp; {len(cards_html)} observations &nbsp;·&nbsp; 10 days history + 2 days forward</p>
<div class="note">⚠️ Saturation is initialized at 100% at the start of each 10-day window. Focus on the trend and where it lands on the visit day, not the absolute starting value.</div>
{''.join(cards_html)}
</body>
</html>'''

out = os.path.join(SCRIPT_DIR, 'validation_report.html')
with open(out, 'w', encoding='utf-8') as f:
    f.write(html)

print(f'\nDone → {out}')
