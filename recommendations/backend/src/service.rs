use crate::error::AppError;
use crate::eval::{evaluate, EvalRow};
use crate::model::Model;
use crate::recommender::{by_name, PopularityRecommender, Recommender};
use crate::seed;
use crate::store::Store;
use crate::types::{NewTitle, NewUser, Profile, Scored, Title, User};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub const ROW_SIZE: usize = 10;

struct Data {
    titles: Vec<Title>,
    users: HashMap<i32, User>,
    histories: HashMap<i32, Vec<i32>>,
    model: Model,
}

pub struct Service<S: Store> {
    store: S,
    data: RwLock<Data>,
    eval: Vec<EvalRow>,
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub title: Title,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct Row {
    pub kind: &'static str,
    pub label: String,
    pub anchor: Option<i32>,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize)]
pub struct Recommendations {
    pub algorithm: &'static str,
    pub cold_start: bool,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize)]
pub struct TitleView {
    pub title: Title,
    pub watches: u32,
}

#[derive(Debug, Serialize)]
pub struct TitleDetail {
    pub title: Title,
    pub watches: u32,
    pub similar_content: Vec<Item>,
    pub also_watched: Vec<Item>,
}

#[derive(Debug, Serialize)]
pub struct UserView {
    pub user: User,
    pub watch_count: usize,
}

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub user: User,
    pub watched: Vec<Title>,
}

fn histories_from(watches: &[(i32, i32)]) -> HashMap<i32, Vec<i32>> {
    let mut h: HashMap<i32, Vec<i32>> = HashMap::new();
    for (u, t) in watches {
        h.entry(*u).or_default().push(*t);
    }
    h
}

fn known_genres(genres: &[String]) -> Result<(), AppError> {
    match genres.iter().find(|g| seed::genre(g).is_none()) {
        Some(g) => Err(AppError::Invalid(format!("unknown genre {g}"))),
        None => Ok(()),
    }
}

async fn seed_store<S: Store>(store: &S) -> Result<(), AppError> {
    let d = seed::generate();
    let titles = store.insert_titles(&d.titles).await?;
    let users = store.insert_users(&d.users).await?;
    let watches: Vec<(i32, i32)> = d.watches.iter().map(|(u, t)| (users[*u].id, titles[*t].id)).collect();
    store.insert_watches(&watches).await
}

impl<S: Store> Service<S> {
    pub async fn bootstrap(store: S) -> Result<Self, AppError> {
        if store.titles().await?.is_empty() {
            seed_store(&store).await?;
        }
        let titles = store.titles().await?;
        let users: HashMap<i32, User> = store.users().await?.into_iter().map(|u| (u.id, u)).collect();
        let histories = histories_from(&store.watches().await?);
        let model = Model::build(&titles, &histories);
        let user_genres: HashMap<i32, Vec<String>> = users
            .values()
            .filter_map(|u| u.cluster.as_deref().map(|c| (u.id, seed::cluster_genres(c))))
            .collect();
        let eval = evaluate(&titles, &histories, &user_genres);
        let service = Self { store, data: RwLock::new(Data { titles, users, histories, model }), eval };
        service.persist_embeddings().await?;
        Ok(service)
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, Data>, AppError> {
        self.data.read().map_err(|_| AppError::Poisoned)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, Data>, AppError> {
        self.data.write().map_err(|_| AppError::Poisoned)
    }

    async fn persist_embeddings(&self) -> Result<(), AppError> {
        let items: Vec<(i32, Vec<f32>)> = {
            let d = self.read()?;
            d.model.embeddings.iter().map(|(id, v)| (*id, v.clone())).collect()
        };
        self.store.save_embeddings(&items).await
    }

    fn rebuild(&self) -> Result<(), AppError> {
        let mut d = self.write()?;
        d.model = Model::build(&d.titles, &d.histories);
        Ok(())
    }

    pub fn eval(&self) -> Vec<EvalRow> {
        self.eval.clone()
    }

    pub fn titles(&self) -> Result<Vec<TitleView>, AppError> {
        let d = self.read()?;
        Ok(d.titles.iter().map(|t| TitleView { title: t.clone(), watches: d.model.popularity.count(t.id) }).collect())
    }

    fn items(d: &Data, scored: &[Scored]) -> Vec<Item> {
        scored
            .iter()
            .filter_map(|s| d.model.titles.get(&s.title_id).map(|t| Item { title: t.clone(), score: s.score }))
            .collect()
    }

    async fn similar_by_vector(&self, query: Vec<f32>, exclude: &HashSet<i32>, k: usize) -> Result<Vec<Item>, AppError> {
        let hits = self.store.nearest(&query, k + exclude.len()).await?;
        let scored: Vec<Scored> = hits
            .into_iter()
            .filter(|(id, _)| !exclude.contains(id))
            .take(k)
            .map(|(title_id, distance)| Scored { title_id, score: 1.0 - distance })
            .collect();
        Ok(Self::items(&*self.read()?, &scored))
    }

    pub async fn title(&self, id: i32) -> Result<TitleDetail, AppError> {
        let (title, watches, query, also) = {
            let d = self.read()?;
            let title = d.model.titles.get(&id).cloned().ok_or_else(|| AppError::NotFound(format!("title {id}")))?;
            let query = d.model.embedding(id).map(|v| v.to_vec()).unwrap_or_default();
            let neighbors: Vec<Scored> =
                d.model.cf.neighbors(id).iter().take(ROW_SIZE).map(|(title_id, score)| Scored { title_id: *title_id, score: *score }).collect();
            (title, d.model.popularity.count(id), query, Self::items(&d, &neighbors))
        };
        let similar_content = self.similar_by_vector(query, &HashSet::from([id]), ROW_SIZE).await?;
        Ok(TitleDetail { title, watches, similar_content, also_watched: also })
    }

    pub async fn create_title(&self, mut new: NewTitle) -> Result<Title, AppError> {
        if new.name.trim().is_empty() || new.genres.is_empty() {
            return Err(AppError::Invalid("a title needs a name and at least one genre".into()));
        }
        known_genres(&new.genres)?;
        if self.read()?.titles.iter().any(|t| t.name.eq_ignore_ascii_case(new.name.trim())) {
            return Err(AppError::Invalid(format!("title {} already exists", new.name)));
        }
        new.name = new.name.trim().to_string();
        new.is_new = true;
        let title = self.store.insert_titles(&[new]).await?.remove(0);
        self.write()?.titles.push(title.clone());
        self.rebuild()?;
        self.persist_embeddings().await?;
        Ok(title)
    }

    pub fn users(&self) -> Result<Vec<UserView>, AppError> {
        let d = self.read()?;
        let mut out: Vec<UserView> = d
            .users
            .values()
            .map(|u| UserView { user: u.clone(), watch_count: d.histories.get(&u.id).map(|h| h.len()).unwrap_or(0) })
            .collect();
        out.sort_by_key(|u| std::cmp::Reverse(u.user.id));
        Ok(out)
    }

    pub fn user(&self, id: i32) -> Result<UserDetail, AppError> {
        let d = self.read()?;
        let user = d.users.get(&id).cloned().ok_or_else(|| AppError::NotFound(format!("user {id}")))?;
        let watched = d
            .histories
            .get(&id)
            .map(|h| h.iter().rev().filter_map(|t| d.model.titles.get(t).cloned()).collect())
            .unwrap_or_default();
        Ok(UserDetail { user, watched })
    }

    pub async fn create_user(&self, name: String, genres: Vec<String>) -> Result<User, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Invalid("a user needs a name".into()));
        }
        known_genres(&genres)?;
        let new = NewUser { name: name.trim().to_string(), cluster: None, picked_genres: genres };
        let user = self.store.insert_users(&[new]).await?.remove(0);
        self.write()?.users.insert(user.id, user.clone());
        Ok(user)
    }

    pub async fn add_watch(&self, user_id: i32, title_id: i32) -> Result<(), AppError> {
        {
            let d = self.read()?;
            if !d.users.contains_key(&user_id) {
                return Err(AppError::NotFound(format!("user {user_id}")));
            }
            if !d.model.titles.contains_key(&title_id) {
                return Err(AppError::NotFound(format!("title {title_id}")));
            }
            if d.histories.get(&user_id).is_some_and(|h| h.contains(&title_id)) {
                return Ok(());
            }
        }
        self.store.insert_watches(&[(user_id, title_id)]).await?;
        self.write()?.histories.entry(user_id).or_default().push(title_id);
        self.rebuild()
    }

    pub async fn recommendations(&self, user_id: i32, algo: &str) -> Result<Recommendations, AppError> {
        let recommender = by_name(algo).ok_or_else(|| AppError::Invalid(format!("unknown algorithm {algo}")))?;
        let (profile, mut rows, similar_query, similar_label) = {
            let d = self.read()?;
            let user = d.users.get(&user_id).ok_or_else(|| AppError::NotFound(format!("user {user_id}")))?;
            let profile = Profile { watched: d.histories.get(&user_id).cloned().unwrap_or_default(), genres: user.picked_genres.clone() };
            let mut rows = vec![Row {
                kind: "top",
                label: format!("Top picks for you ({})", recommender.name()),
                anchor: None,
                items: Self::items(&d, &recommender.recommend(&d.model, &profile, ROW_SIZE)),
            }];
            if let Some(last) = profile.watched.last() {
                rows.push(self.because_row(&d, &profile, *last));
            }
            let (query, label) = match profile.watched.iter().rev().nth(1).or(profile.watched.last()) {
                Some(id) => (d.model.embedding(*id).map(|v| v.to_vec()), (format!("Similar to {}", d.model.titles[id].name), Some(*id))),
                None if !profile.genres.is_empty() => (
                    Some(d.model.embedder.embed_genres(&profile.genres)),
                    (format!("Similar to your picks: {}", profile.genres.join(", ")), None),
                ),
                None => (None, (String::new(), None)),
            };
            (profile, rows, query, label)
        };
        let exclude: HashSet<i32> = profile.watched.iter().copied().collect();
        if let Some(query) = similar_query {
            let items = self.similar_by_vector(query, &exclude, ROW_SIZE).await?;
            rows.push(Row { kind: "similar", label: similar_label.0, anchor: similar_label.1, items });
        }
        {
            let d = self.read()?;
            rows.push(Row {
                kind: "popular",
                label: "Popular right now".into(),
                anchor: None,
                items: Self::items(&d, &PopularityRecommender.recommend(&d.model, &profile, ROW_SIZE)),
            });
        }
        Ok(Recommendations { algorithm: recommender.name(), cold_start: profile.watched.is_empty(), rows })
    }

    fn because_row(&self, d: &Data, profile: &Profile, anchor: i32) -> Row {
        let seen: HashSet<i32> = profile.watched.iter().copied().collect();
        let scored: Vec<Scored> = d
            .model
            .cf
            .neighbors(anchor)
            .iter()
            .filter(|(id, _)| !seen.contains(id))
            .take(ROW_SIZE)
            .map(|(title_id, score)| Scored { title_id: *title_id, score: *score })
            .collect();
        Row {
            kind: "because",
            label: format!("Because you watched {}", d.model.titles[&anchor].name),
            anchor: Some(anchor),
            items: Self::items(d, &scored),
        }
    }
}
