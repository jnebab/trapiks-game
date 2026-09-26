use crate::map::GeoOrigin;

const EARTH_RADIUS_M: f64 = 6_371_008.8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub lat0: f64,
    pub lon0: f64,
    cos_lat0: f64,
}

impl Projection {
    pub fn centered(min_lat: f64, min_lon: f64, max_lat: f64, max_lon: f64) -> Self {
        Self::at((min_lat + max_lat) / 2.0, (min_lon + max_lon) / 2.0)
    }

    pub fn from_origin(origin: GeoOrigin) -> Self {
        Self::at(origin.lat, origin.lon)
    }

    fn at(lat0: f64, lon0: f64) -> Self {
        Self {
            lat0,
            lon0,
            cos_lat0: libm::cos(lat0.to_radians()),
        }
    }

    pub fn project(&self, lat: f64, lon: f64) -> (f64, f64) {
        let x = EARTH_RADIUS_M * (lon - self.lon0).to_radians() * self.cos_lat0;
        let y = -EARTH_RADIUS_M * (lat - self.lat0).to_radians();
        (x, y)
    }

    pub fn origin(&self) -> GeoOrigin {
        GeoOrigin {
            lat: self.lat0,
            lon: self.lon0,
        }
    }
}
