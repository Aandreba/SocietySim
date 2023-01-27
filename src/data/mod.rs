use futures::{stream::FuturesUnordered, FutureExt, Stream, TryFutureExt, TryStreamExt, StreamExt};
use serde::de::DeserializeOwned;
use shared::game_data::{
    building::{RawBuilding, Building},
    good::{Good, RawGood},
    job::{Job, RawJob},
    skill::{RawSkill, Skill},
    Str,
};
use std::{io::BufReader, path::Path, ptr::addr_of};
use tokio::try_join;
use vector_mapp::{r#box::BoxMap, vec::VecMap};

#[derive(Debug)]
pub struct GameContext {
    buildings: BoxMap<Str, Building<'static>>, // depends on goods, jobs & skills
    jobs: BoxMap<Str, Job<'static>>, // depends on skills
    skills: BoxMap<Str, Skill>,
    goods: BoxMap<Str, Good>,
}

impl GameContext {
    #[inline]
    pub fn jobs<'a> (&'a self) -> &'a BoxMap<Str, Job<'a>> {
        return unsafe { &*addr_of!(self.jobs).cast() }
    }

    #[inline]
    pub fn buildings<'a> (&'a self) -> &'a BoxMap<Str, Building<'a>> {
        return unsafe { &*addr_of!(self.buildings).cast() }
    }

    #[inline]
    pub fn skills (&self) -> &BoxMap<Str, Skill> {
        return &self.skills
    }

    #[inline]
    pub fn goods (&self) -> &BoxMap<Str, Good> {
        return &self.goods
    }
}

impl GameContext {
    #[inline]
    async fn load_raw(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let (buildings, goods, jobs, skills) = try_join! {
            Self::load_raw_by_name::<RawBuilding>(path.join("buildings")),
            Self::load_raw_by_name::<RawGood>(path.join("goods")),
            Self::load_raw_by_name::<RawJob>(path.join("jobs")),
            Self::load_raw_by_name::<RawSkill>(path.join("skills")),
        }?;

        let (skills, goods) = try_join! {
            skills
                .map_ok(|(key, value)| (key, Skill::from_raw(value)))
                .try_collect::<VecMap<_, _>>(),

            goods
                .map_ok(|(key, value)| (key, Good::from_raw(value)))
                .try_collect::<VecMap<_, _>>()
        }?;

        let skills = skills.into();
        let goods = goods.into();

        unsafe {
            let jobs = jobs
                .map_ok(|(key, value)| Ok((key, core::mem::transmute(Job::from_raw(value, &skills)?))))
                .map(Result::flatten)
                .try_collect::<VecMap<_, _>>()
                .await?
                .into();

            let buildings = buildings
                .map_ok(|(key, value)| Ok((key, core::mem::transmute(Building::from_raw(value, &goods, &jobs, &skills)?))))
                .map(Result::flatten)
                .try_collect::<VecMap<_, _>>()
                .await?
                .into();
                
            return Ok(Self {
                skills,
                goods,
                jobs,
                buildings
            });
        }

    }

    #[inline]
    async fn load_raw_by_name<T: 'static + Send + DeserializeOwned>(
        path: impl AsRef<Path>,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<(Str, T)>>> {
        let mut dir = tokio::fs::read_dir(path).await?;
        let join = FuturesUnordered::new();

        while let Some(entry) = dir.next_entry().await? {
            if entry.metadata().await?.is_file() {
                let path = entry.path();
                let fut = tokio::task::spawn_blocking(move || {
                    let file = BufReader::new(std::fs::File::open(path)?);
                    return match ron::de::from_reader::<_, VecMap<Str, T>>(file) {
                        Ok(x) => Ok(futures::stream::iter(x.into_iter().map(anyhow::Ok))),
                        Err(e) => Err(anyhow::Error::from(e)),
                    };
                });

                join.push(fut.map_err(anyhow::Error::from).map(Result::flatten));
            }
        }

        return Ok(join.try_flatten());
        //return Ok(join);
    }
}

#[cfg(test)]
mod test {
    use super::GameContext;

    #[tokio::test]
    async fn test () -> anyhow::Result<()> {
        let ctx = GameContext::load_raw("game").await?;
        let buildings = ctx.buildings();

        println!("{buildings:#?}");
        return Ok(())
    }
}
