#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(serde::Serialize))]
#[repr(transparent)]
pub struct GameDuration {
    days: u16
}

impl GameDuration {
    #[inline]
    pub const fn from_days (days: u16) -> Self {
        return Self { days }
    }

    #[inline]
    pub const fn from_weeks (weeks: u16) -> Self {
        return Self::from_days(7 * weeks)
    }

    #[inline]
    pub const fn from_months (months: u16) -> Self {
        return Self::from_days(30 * months)
    }

    #[inline]
    pub const fn from_years (years: u8) -> Self {
        return Self::from_days(365 * (years as u16))
    }
}

#[cfg(not(target_arch = "spirv"))]
impl<'de> serde::Deserialize<'de> for GameDuration {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        todo!()
    }
}