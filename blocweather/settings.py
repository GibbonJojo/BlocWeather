from pathlib import Path
from typing import Optional
import polars as pl

from pydantic import model_validator, ValidationError, TypeAdapter
from pydantic_settings import BaseSettings, SettingsConfigDict, YamlConfigSettingsSource
from blocweather.schema import RegisteredPoint


class APISettings(BaseSettings):
    model_config = SettingsConfigDict(yaml_file="blocweather.yaml")
    data_path: str
    registered_points_file: Optional[str] = None
    registered_points: Optional[list[RegisteredPoint]] = None

    @classmethod
    def settings_customise_sources(cls, settings_cls, **kwargs):
        return (YamlConfigSettingsSource(settings_cls),)

    @model_validator(mode="after")
    def get_registered_points(self):
        if self.registered_points_file:
            rfile = Path(self.registered_points_file)
            if not rfile.is_file():
                raise ValidationError(f"File {self.registered_points_file} not found")
            self.registered_points = TypeAdapter(list[RegisteredPoint]).validate_json(
                rfile.read_bytes()
            )

            pl.DataFrame(self.registered_points).write_parquet(
                f"{self.data_path}/locations.parquet"
            )
        return self


settings = APISettings()

Path(settings.data_path).mkdir(exist_ok=True, parents=True)
