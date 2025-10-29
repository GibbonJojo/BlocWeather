import openmeteo_requests
from datetime import datetime, timedelta

import polars as pl
from copy import deepcopy
import requests_cache
from retry_requests import retry

from blocweather.settings import settings


def _open_meteo_etl(
    openmeteo,
    request_params: dict,
    parameters: list,
    url: str,
    start: datetime | None,
    end: datetime | None,
    forecast_days: int | None,
) -> dict[str, pl.DataFrame]:
    req_params = deepcopy(request_params)
    if start:
        req_params["start_date"] = start.strftime("%Y-%m-%d")
    if end:
        req_params["end_date"] = end.strftime("%Y-%m-%d")
    if forecast_days:
        req_params["forecast_days"] = forecast_days
    responses = openmeteo.weather_api(url, params=req_params)
    return_data = {}

    # Process 2 locations
    for reg_point, response in zip(settings.registered_points, responses):
        # assert reg_point.latitude == response.Latitude()
        # assert reg_point.longitude == response.Longitude()
        print(f"\nCoordinates: {response.Latitude()}°N {response.Longitude()}°E")
        print(f"Elevation: {response.Elevation()} m asl")
        print(f"Timezone difference to GMT+0: {response.UtcOffsetSeconds()}s")
        hourly = response.Hourly()

        data = pl.DataFrame(
            {
                param: hourly.Variables(i).ValuesAsNumpy()
                for i, param in enumerate(parameters)
            }
        ).with_columns(
            pl.datetime_range(
                start=datetime.fromtimestamp(hourly.Time()),
                end=datetime.fromtimestamp(hourly.TimeEnd()),
                interval=timedelta(seconds=hourly.Interval()),
                closed="both" if "archive" in url else "left",
            ).alias("timestamp")
        )

        return_data[reg_point.name] = data
    return return_data


def open_meteo_etl(start: datetime, to_obs: timedelta, to_fcst: timedelta):
    # Setup the Open-Meteo API client with cache and retry on error
    cache_session = requests_cache.CachedSession(".cache", expire_after=-1)
    retry_session = retry(cache_session, retries=5, backoff_factor=0.2)
    openmeteo = openmeteo_requests.Client(session=retry_session)

    latitudes = [point.latitude for point in settings.registered_points]
    longitudes = [point.longitude for point in settings.registered_points]

    # Make sure all required weather variables are listed here
    # The order of variables in hourly or daily is important to assign them correctly below
    parameters = [
        "temperature_2m",
        "relative_humidity_2m",
        "dew_point_2m",
        "precipitation",
        "cloud_cover",
        "et0_fao_evapotranspiration",
        "wind_speed_10m",
        "wind_direction_10m",
        "is_day",
        "sunshine_duration",
        "shortwave_radiation_instant",
    ]

    request_params = {
        "latitude": latitudes,
        "longitude": longitudes,
        "hourly": parameters,
        "timezone": "GMT",
        "wind_speed_unit": "ms",
    }
    print("Loading Data")
    obs_data = _open_meteo_etl(
        openmeteo,
        request_params,
        parameters,
        "https://archive-api.open-meteo.com/v1/archive",
        start - to_obs,
        start,
        None,
    )
    fcst_data = _open_meteo_etl(
        openmeteo,
        request_params,
        parameters,
        "https://api.open-meteo.com/v1/forecast",
        None,
        None,
        None,
    )
    # TODO get distance between requested point and response point
    for point in settings.registered_points:
        pl.concat([obs_data[point.name], fcst_data[point.name]]).unique(
            "timestamp", keep="first"
        ).sort("timestamp").write_parquet(
            f"{settings.data_path}/locations/{point.name}.parquet"
        )


# if __name__ == "__main__":
#     now = datetime.now()
#     open_meteo_etl(now - timedelta(hours=24), now)
