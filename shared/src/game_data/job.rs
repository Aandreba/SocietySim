use super::{
    skill::{NamedSkill, Skill},
    try_get_key_value, NamedEntry, Str,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use vector_mapp::r#box::BoxMap;

pub type NamedJob<'a> = NamedEntry<'a, Job<'a>>;

#[derive(Debug)]
pub struct Job<'a> {
    pub skills: BoxMap<NamedSkill<'a>, f32>,
}

impl<'a> Job<'a> {
    #[inline]
    pub fn from_raw(raw: RawJob, skills: &'a BoxMap<Str, Skill>) -> anyhow::Result<Self> {
        return Ok(Self {
            skills: Self::raw_skills(raw.skills, skills)?,
        });
    }

    fn raw_skills(
        raw: RawJobSkills,
        skills: &'a BoxMap<Str, Skill>,
    ) -> anyhow::Result<BoxMap<NamedSkill<'a>, f32>> {
        return match raw {
            RawJobSkills::Regular(x) => x
                .into_vec()
                .into_iter()
                .map(|x| Ok((try_get_key_value(skills, &x)?, 1.0)))
                .try_collect(),

            RawJobSkills::Weighted(x) => x
                .into_iter()
                .map(|(x, w)| try_get_key_value(skills, &x).map(|x| (x, w)))
                .try_collect(),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawJob {
    pub skills: RawJobSkills,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawJobSkills {
    Regular(Box<[Str]>),
    Weighted(BoxMap<Str, f32>),
}
