use chrono::{Datelike, NaiveDate};

use super::astronomy::{self as astro, HourAngleInput, Interpolated};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

struct SolarCoordinates {
    declination: f64,
    right_ascension: f64,
    apparent_sidereal_time: f64,
}

impl SolarCoordinates {
    fn new(julian_day: f64) -> Self {
        let t = astro::julian_century(julian_day);
        let l0 = astro::mean_solar_longitude(t);
        let lp = astro::mean_lunar_longitude(t);
        let omega = astro::ascending_lunar_node_longitude(t);
        let lambda = astro::apparent_solar_longitude(t, l0).to_radians();
        let theta0 = astro::mean_sidereal_time(t);
        let d_psi = astro::nutation_in_longitude(l0, lp, omega);
        let d_epsilon = astro::nutation_in_obliquity(l0, lp, omega);
        let epsilon0 = astro::mean_obliquity_of_the_ecliptic(t);
        let epsilon = astro::apparent_obliquity_of_the_ecliptic(t, epsilon0).to_radians();
        Self {
            declination: (epsilon.sin() * lambda.sin()).asin().to_degrees(),
            right_ascension: astro::unwind_angle(
                (epsilon.cos() * lambda.sin())
                    .atan2(lambda.cos())
                    .to_degrees(),
            ),
            apparent_sidereal_time: theta0 + d_psi * (epsilon0 + d_epsilon).to_radians().cos(),
        }
    }
}

pub struct SolarTime {
    coordinates: Coordinates,
    approximate_transit: f64,
    sidereal_time: f64,
    right_ascension: Interpolated,
    declination: Interpolated,
    pub transit: f64,
    pub sunrise: f64,
    pub sunset: f64,
}

const SOLAR_ALTITUDE: f64 = -50.0 / 60.0;

impl SolarTime {
    pub fn new(date: NaiveDate, coordinates: Coordinates) -> Self {
        let julian_day = astro::julian_day(date.year(), date.month(), date.day());
        let solar = SolarCoordinates::new(julian_day);
        let previous = SolarCoordinates::new(julian_day - 1.0);
        let next = SolarCoordinates::new(julian_day + 1.0);
        let right_ascension = Interpolated {
            current: solar.right_ascension,
            previous: previous.right_ascension,
            next: next.right_ascension,
        };
        let declination = Interpolated {
            current: solar.declination,
            previous: previous.declination,
            next: next.declination,
        };
        let m0 = astro::approximate_transit(
            coordinates.longitude,
            solar.apparent_sidereal_time,
            solar.right_ascension,
        );
        let mut time = Self {
            coordinates,
            approximate_transit: m0,
            sidereal_time: solar.apparent_sidereal_time,
            transit: astro::corrected_transit(
                m0,
                coordinates.longitude,
                solar.apparent_sidereal_time,
                &right_ascension,
            ),
            right_ascension,
            declination,
            sunrise: f64::NAN,
            sunset: f64::NAN,
        };
        time.sunrise = time.hour_angle(SOLAR_ALTITUDE, false);
        time.sunset = time.hour_angle(SOLAR_ALTITUDE, true);
        time
    }

    pub fn hour_angle(&self, angle: f64, after_transit: bool) -> f64 {
        astro::corrected_hour_angle(
            &HourAngleInput {
                m0: self.approximate_transit,
                latitude: self.coordinates.latitude,
                longitude: self.coordinates.longitude,
                sidereal_time: self.sidereal_time,
                right_ascension: &self.right_ascension,
                declination: &self.declination,
            },
            angle,
            after_transit,
        )
    }

    pub fn afternoon(&self, shadow_length: f64) -> f64 {
        let tangent = (self.coordinates.latitude - self.declination.current).abs();
        let inverse = shadow_length + tangent.to_radians().tan();
        self.hour_angle((1.0 / inverse).atan().to_degrees(), true)
    }
}
