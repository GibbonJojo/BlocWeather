from pydantic import BaseModel


class RegisteredPoint(BaseModel):
    name: str
    latitude: float
    longitude: float
    rock_type: str
    country: str
    subgroup: str
