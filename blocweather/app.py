from flask import Flask, jsonify, render_template
from flask_cors import CORS
from datetime import datetime, timedelta, timezone
from blocweather.settings import settings
from pathlib import Path
import polars as pl
import os
import logging

logging.basicConfig(level=logging.INFO, filename='app.log',
                    format='%(asctime)s %(levelname)s:%(message)s')

# If you build the frontend and copy dist into backend/frontend_dist,
# Flask will serve it as static files.
FRONTEND_DIST = os.path.join(os.path.dirname(__file__), 'frontend/dist')


app = Flask(__name__, static_folder = FRONTEND_DIST, template_folder = FRONTEND_DIST, static_url_path='')
# Enable CORS for development. In production consider limiting origins.
CORS(app)


@app.route('/')
def index():
    return render_template("index.html")


@app.route('/api/timeseries/<location>/<parameter>')
def timeseries(location: str, parameter: str, days_obs: int = 5, days_fcst: int = 3):
    logging.info(f"Received request for location: {location}, parameter: {parameter}")
    data_file = Path(settings.data_path) / "locations" / f"{location}.parquet"
    if not data_file.is_file():
        return "ERROR"
    now = datetime.now(tz=timezone.utc)
    start = now - timedelta(days=days_obs)
    end = now + timedelta(days=days_fcst)
    logging.info(f"Fetching data from {start} to {end}")
    data = (
        pl.scan_parquet(data_file).select(["timestamp", parameter])
        .with_columns(pl.col("timestamp").dt.replace_time_zone("UTC"))
        .filter((pl.col("timestamp") > start) & (pl.col("timestamp") < end))
        .with_columns(pl.col("timestamp").dt.replace_time_zone(None))
        .rename({"timestamp": "x", parameter: "y"})
    )
    logging.info("Collecting Data")

    data = data.collect()
    logging.info("Converting Data")
    data = data.to_dicts()
    logging.info(f"Retrieved {len(data)} data points")

    # data = [{"x": d["x"], "y": round(d["y"], 1)} for d in data]
    return jsonify(data)



#
# # Serve the built frontend (SPA). If no built frontend exists, this will 404 for index.
# @app.route('/', defaults={'path': ''})
# @app.route('/<path:path>')
# def serve_frontend(path):
# # If path maps to a static file, serve it
# if path != '' and os.path.exists(os.path.join(FRONTEND_DIST, path)):
# return send_from_directory(FRONTEND_DIST, path)
# # otherwise, serve index.html (SPA entrypoint)
# return send_from_directory(FRONTEND_DIST, 'index.html')
#
#


if __name__ == '__main__':
    # For local dev
    app.run(host='0.0.0.0', port=5000, debug=True)