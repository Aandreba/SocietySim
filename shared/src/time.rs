use core::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(serde::Serialize, serde::Deserialize), serde(from = "GameDurationDeser"))]
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
    pub const fn as_days(self) -> u16 {
        return self.days;
    }

    #[inline]
    pub const fn as_weeks(self) -> u16 {
        return self.days / 7;
    }

    #[inline]
    pub fn as_weeks_f32(self) -> f32 {
        return (self.days as f32) / 7f32;
    }

    #[inline]
    pub const fn as_months(self) -> u16 {
        return self.days / 30;
    }

    #[inline]
    pub fn as_months_f32(self) -> f32 {
        return (self.days as f32) / 30f32;
    }

    #[inline]
    pub const fn as_years(self) -> u8 {
        return (self.days / 365) as u8;
    }

    #[inline]
    pub fn as_years_f32(self) -> f32 {
        return (self.days as f32) / 365f32;
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum GameDurationDeser {
    Days (u16),
    Parts {
        #[serde(default)]
        days: u16,
        #[serde(default)]
        weeks: u16,
        #[serde(default)]
        months: u16,
        #[serde(default)]
        years: u8
    }
}

#[cfg(not(target_arch = "spirv"))]
impl From<GameDurationDeser> for GameDuration {
    #[inline]
    fn from(value: GameDurationDeser) -> Self {
        match value {
            GameDurationDeser::Days(days) => GameDuration::from_days(days),
            GameDurationDeser::Parts { days, weeks, months, years  } => GameDuration::new(days, weeks, months, years)
        }
    }
}