use super::{good::{NamedGood, Good}, job::{NamedJob, Job}, skill::{NamedSkill, Skill}, Str, NamedEntry};
use core::num::NonZeroU32;
use serde::{Deserialize, Serialize};
use vector_mapp::{r#box::BoxMap, vec::VecMap};

pub type NamedBuilding<'a> = NamedEntry<'a, Building<'a>>;

#[derive(Debug)]
#[repr(C)]
pub struct Building<'a> {
    pub consumtion: BoxMap<NamedGood<'a>, u32>,
    pub production: BoxMap<NamedGood<'a>, u32>,
    pub jobs: BoxMap<NamedJob<'a>, BuildingJob<'a>>,
}

impl<'a> Building<'a> {
    pub fn from_raw(raw: RawBuilding, goods: &'a BoxMap<Str, Good>, jobs: &'a BoxMap<Str, Job<'a>>, skills: &'a BoxMap<Str, Skill>) -> Self {
        return Self {
            consumtion: raw.consumption.into_iter().map(|(key, value)| goods.get_key_value(&key)),
            production: todo!(),
            jobs: raw.jobs.into_iter()
                .filter_map(|(job, data)| {
                    let job = jobs.get_key_value(&job)?.into();
                    let data = BuildingJob::from_raw(data, skills);
                    return Some((job, data))
                })
                .collect::<VecMap<_, _>>()
                .into(),
        }
    }
}

#[derive(Debug)]
pub struct BuildingJob<'a> {
    pub amount: NonZeroU32,
    pub learned_skills: Box<[NamedSkill<'a>]>,
}

impl<'a> BuildingJob<'a> {
    pub fn from_raw(raw: RawBuildingJob, skills: &'a BoxMap<Str, Skill>) -> Self {
        return match raw {
            RawBuildingJob::Regular(amount) => Self {
                amount,
                learned_skills: Box::default(),
            },
            RawBuildingJob::Advanced {
                amount,
                learned_skills,
            } => Self {
                amount,
                learned_skills: learned_skills
                    .into_vec()
                    .into_iter()
                    .filter_map(|x| skills.get_key_value(&x))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawBuilding {
    pub consumption: BoxMap<Str, u32>,
    pub production: BoxMap<Str, u32>,
    pub jobs: BoxMap<Str, RawBuildingJob>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawBuildingJob {
    Regular(NonZeroU32),
    Advanced {
        amount: NonZeroU32,
        /// Skills that will be learned in the job
        learned_skills: Box<[Str]>,
    },
}
