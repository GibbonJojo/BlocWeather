from blocweather.etl.etl import open_meteo_etl
from datetime import datetime, timedelta


if __name__ == "__main__":
    now = datetime.now()
    open_meteo_etl(now, timedelta(days=7), timedelta(hours=48))
