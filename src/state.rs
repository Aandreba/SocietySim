use shared::{time::GameDuration, game_data::{building::NamedBuilding, job::NamedJob, skill::{NamedSkill}, good::{NamedGood, Good}}};
use vector_mapp::{vec::VecMap, r#box::BoxMap};

type Str = Box<str>;
type Market<'a> = BoxMap<NamedGood<'a>, Availability>;

pub struct GameState<'a> {
    date: GameDuration,
    market: Market<'a>
}

impl GameState<'_> {
    pub fn tick (&mut self) {
        // Calculate births
        // Calculate deaths

        // Calculate production
        // Calculate layoffs
        // Calculate hirings
    }
}

pub enum WorkStatus<'a> {
    Unemployed,
    Working (&'a GameBuilding<'a>, NamedJob<'a>),
    Studying (&'a GameBuilding<'a>)
}

pub struct Person<'a> {
    age: GameDuration,
    intelligence: u8,
    health: u8,
    work: WorkStatus<'a>,
    skills: VecMap<NamedSkill<'a>, u8>
}

impl Person<'_> {
    #[inline]
    pub fn job_productivity (&self) -> Option<f32> {
        if let WorkStatus::Working(_, job) = self.work {
            let mut result = 0.0;
            for (skill, weight) in job.value.skills.iter() {
                if let Some(value) = self.skills.get(&skill).copied() {
                    result += weight * value as f32;
                }
            }
            return Some(result)
        }
        return None
    }
}

pub struct GameBuilding<'a> {
    ty: NamedBuilding<'a>,
    cash_reserves: f32,
    workers: Vec<&'a Person<'a>>,
    students: Vec<&'a Person<'a>>
}

impl GameBuilding<'_> {
    pub fn tick (&self) {
        let productivity = self.workers.iter().copied().filter_map(Person::job_productivity).sum::<f32>();
    }

    #[inline]
    pub fn consumption_cost (&self, market: &Market) -> f32 {
        self.ty.value.consumtion.iter().map(|(good, count)| match market.get_key_value(good) {
            Some((good, av)) => av.price(good.value) * count,
            None => f32::INFINITY
        }).sum()
    }

    #[inline]
    pub fn production_earnings (&self, market: &Market) -> f32 {
        self.ty.value.consumtion.iter().map(|(good, count)| match market.get_key_value(good) {
            Some((good, av)) => av.price(good.value) * count,
            None => f32::INFINITY
        }).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Availability {
    supply: f32,
    demand: f32
}

impl Availability {
    #[inline]
    pub fn price (self, good: &Good) -> f32 {
        return good.base_cost * self.demand / self.supply
    }
}