use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(serde::Serialize))]
#[repr(transparent)]
pub struct GameDuration {
    days: u16,
}

impl GameDuration {
    #[inline]
    pub const fn new(days: u16, weeks: u16, months: u16, years: u8) -> Self {
        return Self::from_days(days + (7 * weeks) + (30 * months) + (365 * years as u16));
    }

    #[inline]
    pub const fn from_days(days: u16) -> Self {
        return Self { days };
    }

    #[inline]
    pub const fn from_weeks(weeks: u16) -> Self {
        return Self::from_days(7 * weeks);
    }

    #[inline]
    pub const fn from_months(months: u16) -> Self {
        return Self::from_days(30 * months);
    }

    #[inline]
    pub const fn from_years(years: u8) -> Self {
        return Self::from_days(365 * (years as u16));
    }

    #[inline]
    pub const fn as_days (self) -> u16 {
        return self.days
    }

    #[inline]
    pub const fn as_weeks (self) -> u16 {
        return self.days / 7
    }

    #[inline]
    pub fn as_weeks_f32 (self) -> f32 {
        return (self.days as f32) / 7f32
    }

    #[inline]
    pub const fn as_months (self) -> u16 {
        return self.days / 30
    }

    #[inline]
    pub fn as_months_f32 (self) -> f32 {
        return (self.days as f32) / 30f32
    }

    #[inline]
    pub const fn as_years (self) -> u8 {
        return (self.days / 365) as u8
    }

    #[inline]
    pub fn as_years_f32 (self) -> f32 {
        return (self.days as f32) / 365f32
    }
}

impl Add for GameDuration {
    type Output = GameDuration;

    #[inline]
    fn add(mut self, rhs: Self) -> Self::Output {
        self.days += rhs.days;
        self
    }
}

impl Sub for GameDuration {
    type Output = GameDuration;

    #[inline]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self.days -= rhs.days;
        self
    }
}

impl AddAssign for GameDuration {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.days += rhs.days
    }
}

impl SubAssign for GameDuration {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.days -= rhs.days
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<'de> serde::Deserialize<'de> for GameDuration {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LocalVisitor;
        impl<'de> serde::de::Visitor<'de> for LocalVisitor {
            type Value = GameDuration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a number of days or a definition")
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                return Ok(GameDuration::from_days(v));
            }

            #[inline]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut days = None;
                let mut weeks = None;
                let mut months = None;
                let mut years = None;

                loop {
                    match map.next_key::<&'de str>()? {
                        Some("days") if days.is_none() => {
                            days = map.next_value::<u16>().map(Some)?
                        }
                        Some("weeks") if weeks.is_none() => {
                            weeks = map.next_value::<u16>().map(Some)?
                        }
                        Some("months") if months.is_none() => {
                            months = map.next_value::<u16>().map(Some)?
                        }
                        Some("years") if years.is_none() => {
                            years = map.next_value::<u8>().map(Some)?
                        }
                        Some(key @ ("days" | "weeks" | "months" | "years")) => {
                            return Err(serde::de::Error::custom(format_args!("duplicate field `{key}`")))
                        }
                        Some(other) => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["days", "weeks", "months", "years"],
                            ))
                        }
                        None => break,
                    }
                }

                return Ok(GameDuration::new(
                    days.unwrap_or_default(),
                    weeks.unwrap_or_default(),
                    months.unwrap_or_default(),
                    years.unwrap_or_default(),
                ));
            }
        }

        return deserializer.deserialize_any(LocalVisitor)
    }
}
