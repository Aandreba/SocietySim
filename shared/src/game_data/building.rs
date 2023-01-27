use super::{
    good::{Good, NamedGood},
    job::{Job, NamedJob},
    skill::{NamedSkill, Skill},
    try_get_key_value, NamedEntry, Str,
};
use core::num::NonZeroU32;
use serde::{Deserialize, Serialize};
use vector_mapp::r#box::BoxMap;

pub type NamedBuilding<'a> = NamedEntry<'a, Building<'a>>;

#[derive(Debug)]
#[repr(C)]
pub struct Building<'a> {
    pub consumtion: BoxMap<NamedGood<'a>, u32>,
    pub production: BoxMap<NamedGood<'a>, u32>,
    pub jobs: BoxMap<NamedJob<'a>, BuildingJob<'a>>,
    pub learned_skills: Option<BoxMap<&'a Str, &'a Skill>>,
}

impl<'a> Building<'a> {
    pub fn from_raw(
        raw: RawBuilding,
        goods: &'a BoxMap<Str, Good>,
        jobs: &'a BoxMap<Str, Job<'a>>,
        skills: &'a BoxMap<Str, Skill>,
    ) -> anyhow::Result<Self> {
        return Ok(Self {
            consumtion: raw
                .consumption
                .into_iter()
                .map(|(key, value)| anyhow::Ok((try_get_key_value(goods, &key)?, value)))
                .try_collect::<BoxMap<_, _>>()?,

            production: raw
                .production
                .into_iter()
                .map(|(key, value)| anyhow::Ok((try_get_key_value(goods, &key)?, value)))
                .try_collect::<BoxMap<_, _>>()?,

            jobs: raw
                .jobs
                .into_iter()
                .map(|(job, data)| {
                    let job = try_get_key_value(jobs, &job)?;
                    let data = BuildingJob::from_raw(data, skills);
                    return anyhow::Ok((job, data));
                })
                .try_collect::<BoxMap<_, _>>()?,

            learned_skills: match raw.learned_skills {
                Some(learned_skills) => Some(learned_skills
                    .into_vec()
                    .into_iter()
                    .map(|x| try_get_key_value(skills, &x).map(Into::into))
                    .try_collect()?),
                None => None
            },
        });
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
                    .filter_map(|x| skills.get_key_value(&x).map(Into::into))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            },
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawBuilding {
    pub consumption: BoxMap<Str, u32>,
    #[serde(default)]
    pub production: BoxMap<Str, u32>,
    pub jobs: BoxMap<Str, RawBuildingJob>,
    #[serde(default)]
    pub learned_skills: Option<Box<[Str]>>,
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
