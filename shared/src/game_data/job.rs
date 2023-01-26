use super::{
    skill::{NamedSkill, Skill},
    Str, NamedEntry,
};
use elor::Either;
use serde::{Deserialize, Serialize};
use vector_mapp::r#box::BoxMap;

pub type NamedJob<'a> = NamedEntry<'a, Job<'a>>;

#[derive(Debug)]
pub struct Job<'a> {
    pub skills: Box<[JobSkill<'a>]>,
}

impl<'a> Job<'a> {
    #[inline]
    pub fn from_raw(raw: RawJob, skills: &'a BoxMap<Str, Skill>) -> Self {
        return Self {
            skills: JobSkill::from_raw(raw.skills, skills).collect(),
        }
    }
}

#[derive(Debug)]
pub struct JobSkill<'a> {
    pub skill: NamedSkill<'a>,
    pub weight: f32,
}

impl<'a> JobSkill<'a> {
    pub fn from_raw(
        raw: RawJobSkills,
        skills: &'a BoxMap<Str, Skill>,
    ) -> impl Iterator<Item = JobSkill<'a>> {
        return match raw {
            RawJobSkills::Regular(x) => {
                Either::Left(x.into_vec().into_iter()
                    .filter_map(|x| skills.get_key_value(&x))
                    .map(|skill| Self { skill: skill.into(), weight: 1.0 })
                ).into_same_iter()
            }

            RawJobSkills::Weighted(x) => Either::Right(
                x.into_iter()
                    .filter_map(|(skill, weight)| skills.get_key_value(&skill).map(|skill| Self { skill: skill.into(), weight }))
            ).into_same_iter(),
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
