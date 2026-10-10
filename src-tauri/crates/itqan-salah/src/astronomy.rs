pub fn unwind_angle(angle: f64) -> f64 {
    normalize_to_scale(angle, 360.0)
}

pub fn normalize_to_scale(value: f64, max: f64) -> f64 {
    value - max * (value / max).floor()
}

fn quadrant_shift_angle(angle: f64) -> f64 {
    if (-180.0..=180.0).contains(&angle) {
        angle
    } else {
        angle - 360.0 * (angle / 360.0).round()
    }
}

fn sin_deg(degrees: f64) -> f64 {
    degrees.to_radians().sin()
}

fn cos_deg(degrees: f64) -> f64 {
    degrees.to_radians().cos()
}

pub fn julian_day(year: i32, month: u32, day: u32) -> f64 {
    let (y, m) = if month > 2 {
        (f64::from(year), f64::from(month))
    } else {
        (f64::from(year - 1), f64::from(month + 12))
    };
    let a = (y / 100.0).trunc();
    let b = (2.0 - a + (a / 4.0).trunc()).trunc();
    (365.25 * (y + 4716.0)).trunc() + (30.6001 * (m + 1.0)).trunc() + f64::from(day) + b - 1524.5
}

pub fn julian_century(julian_day: f64) -> f64 {
    (julian_day - 2_451_545.0) / 36_525.0
}

pub fn mean_solar_longitude(t: f64) -> f64 {
    unwind_angle(280.466_456_7 + 36_000.769_83 * t + 0.000_303_2 * t.powi(2))
}

pub fn mean_lunar_longitude(t: f64) -> f64 {
    unwind_angle(218.3165 + 481_267.881_3 * t)
}

pub fn ascending_lunar_node_longitude(t: f64) -> f64 {
    unwind_angle(125.044_52 - 1_934.136_261 * t + 0.002_070_8 * t.powi(2) + t.powi(3) / 450_000.0)
}

fn mean_solar_anomaly(t: f64) -> f64 {
    unwind_angle(357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t.powi(2))
}

fn solar_equation_of_the_center(t: f64, mean_anomaly: f64) -> f64 {
    let m = mean_anomaly.to_radians();
    (1.914_602 - 0.004_817 * t - 0.000_014 * t.powi(2)) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin()
}

pub fn apparent_solar_longitude(t: f64, mean_longitude: f64) -> f64 {
    let longitude = mean_longitude + solar_equation_of_the_center(t, mean_solar_anomaly(t));
    let omega = 125.04 - 1934.136 * t;
    unwind_angle(longitude - 0.005_69 - 0.004_78 * sin_deg(omega))
}

pub fn mean_obliquity_of_the_ecliptic(t: f64) -> f64 {
    23.439_291 - 0.013_004_167 * t - 0.000_000_163_9 * t.powi(2) + 0.000_000_503_6 * t.powi(3)
}

pub fn apparent_obliquity_of_the_ecliptic(t: f64, mean_obliquity: f64) -> f64 {
    mean_obliquity + 0.002_56 * cos_deg(125.04 - 1934.136 * t)
}

pub fn mean_sidereal_time(t: f64) -> f64 {
    let jd = t * 36_525.0 + 2_451_545.0;
    unwind_angle(
        280.460_618_37 + 360.985_647_366_29 * (jd - 2_451_545.0) + 0.000_387_933 * t.powi(2)
            - t.powi(3) / 38_710_000.0,
    )
}

pub fn nutation_in_longitude(solar: f64, lunar: f64, node: f64) -> f64 {
    -17.2 / 3600.0 * sin_deg(node)
        - 1.32 / 3600.0 * sin_deg(2.0 * solar)
        - 0.23 / 3600.0 * sin_deg(2.0 * lunar)
        + 0.21 / 3600.0 * sin_deg(2.0 * node)
}

pub fn nutation_in_obliquity(solar: f64, lunar: f64, node: f64) -> f64 {
    9.2 / 3600.0 * cos_deg(node)
        + 0.57 / 3600.0 * cos_deg(2.0 * solar)
        + 0.1 / 3600.0 * cos_deg(2.0 * lunar)
        - 0.09 / 3600.0 * cos_deg(2.0 * node)
}

fn altitude_of_celestial_body(latitude: f64, declination: f64, hour_angle: f64) -> f64 {
    (sin_deg(latitude) * sin_deg(declination)
        + cos_deg(latitude) * cos_deg(declination) * cos_deg(hour_angle))
    .asin()
    .to_degrees()
}

pub fn approximate_transit(longitude: f64, sidereal_time: f64, right_ascension: f64) -> f64 {
    let m0 = normalize_to_scale((right_ascension - longitude - sidereal_time) / 360.0, 1.0);
    let expected = normalize_to_scale((12.0 - longitude / 15.0) / 24.0, 1.0);
    if m0 - expected > 0.5 {
        m0 - 1.0
    } else if expected - m0 > 0.5 {
        m0 + 1.0
    } else {
        m0
    }
}

pub struct Interpolated {
    pub current: f64,
    pub previous: f64,
    pub next: f64,
}

impl Interpolated {
    fn value(&self, n: f64) -> f64 {
        let a = self.current - self.previous;
        let b = self.next - self.current;
        self.current + n / 2.0 * (a + b + n * (b - a))
    }

    fn angle(&self, n: f64) -> f64 {
        let a = unwind_angle(self.current - self.previous);
        let b = unwind_angle(self.next - self.current);
        self.current + n / 2.0 * (a + b + n * (b - a))
    }
}

pub fn corrected_transit(
    m0: f64,
    longitude: f64,
    sidereal_time: f64,
    right_ascension: &Interpolated,
) -> f64 {
    let theta = unwind_angle(sidereal_time + 360.985_647 * m0);
    let a = unwind_angle(right_ascension.angle(m0));
    let h = quadrant_shift_angle(theta + longitude - a);
    (m0 - h / 360.0) * 24.0
}

pub struct HourAngleInput<'a> {
    pub m0: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub sidereal_time: f64,
    pub right_ascension: &'a Interpolated,
    pub declination: &'a Interpolated,
}

pub fn corrected_hour_angle(input: &HourAngleInput, angle: f64, after_transit: bool) -> f64 {
    let latitude = input.latitude;
    let d2 = input.declination.current;
    let h0 = ((sin_deg(angle) - sin_deg(latitude) * sin_deg(d2))
        / (cos_deg(latitude) * cos_deg(d2)))
    .acos()
    .to_degrees();
    let m = if after_transit {
        input.m0 + h0 / 360.0
    } else {
        input.m0 - h0 / 360.0
    };
    let theta = unwind_angle(input.sidereal_time + 360.985_647 * m);
    let a = unwind_angle(input.right_ascension.angle(m));
    let delta = input.declination.value(m);
    let hour_angle = theta + input.longitude - a;
    let altitude = altitude_of_celestial_body(latitude, delta, hour_angle);
    let dm =
        (altitude - angle) / (360.0 * cos_deg(delta) * cos_deg(latitude) * sin_deg(hour_angle));
    (m + dm) * 24.0
}
