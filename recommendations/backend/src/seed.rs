use crate::rng::Rng;
use crate::types::{NewTitle, NewUser, Title};
use std::collections::{HashMap, HashSet};

pub const SEED: u64 = 15;
pub const TITLE_COUNT: usize = 100;
pub const USER_COUNT: usize = 500;

pub struct GenreInfo {
    pub name: &'static str,
    pub tags: [&'static str; 4],
    pub roles: [&'static str; 2],
    pub settings: [&'static str; 2],
    pub goals: [&'static str; 2],
}

pub const GENRES: [GenreInfo; 16] = [
    GenreInfo { name: "Action", tags: ["tournament", "swordplay", "martial arts", "rivalry"], roles: ["a hot-blooded fighter", "a disgraced swordsman"], settings: ["a city ruled by warring clans", "an underground arena"], goals: ["to become the strongest", "to avenge a fallen master"] },
    GenreInfo { name: "Adventure", tags: ["quest", "journey", "treasure", "found family"], roles: ["a restless wanderer", "a young cartographer"], settings: ["uncharted sky islands", "a desert of buried ruins"], goals: ["to find a legendary treasure", "to map the edge of the world"] },
    GenreInfo { name: "Comedy", tags: ["gag", "parody", "chaotic roommates", "school club"], roles: ["a hopelessly clumsy student", "a lazy part-time worker"], settings: ["a dorm full of eccentrics", "a failing convenience store"], goals: ["to survive one normal week", "to save the club from closing"] },
    GenreInfo { name: "Drama", tags: ["tragedy", "coming of age", "family secrets", "redemption"], roles: ["a grieving older sibling", "a former prodigy"], settings: ["a fading seaside town", "a hospital ward at night"], goals: ["to mend a broken family", "to find a reason to keep going"] },
    GenreInfo { name: "Fantasy", tags: ["magic academy", "dragons", "ancient prophecy", "spirits"], roles: ["an apprentice mage", "a dragon tamer"], settings: ["a floating magic academy", "a kingdom of ancient spirits"], goals: ["to fulfill an ancient prophecy", "to master forbidden spells"] },
    GenreInfo { name: "Sci-Fi", tags: ["space colony", "time loop", "cyberpunk", "artificial intelligence"], roles: ["a rogue android", "a colony engineer"], settings: ["a dying orbital colony", "a neon megacity"], goals: ["to escape an endless time loop", "to awaken a sleeping starship"] },
    GenreInfo { name: "Romance", tags: ["love triangle", "childhood friends", "slow burn", "confession"], roles: ["a shy florist", "a transfer student"], settings: ["a quiet suburban school", "a rainy flower shop"], goals: ["to confess before graduation", "to win back a childhood friend"] },
    GenreInfo { name: "Slice of Life", tags: ["cafe", "countryside", "cooking", "everyday school"], roles: ["a retired office worker", "a curious first-year"], settings: ["a sleepy countryside cafe", "a small mountain village"], goals: ["to cook the perfect bowl of ramen", "to enjoy one peaceful summer"] },
    GenreInfo { name: "Mystery", tags: ["detective", "locked room", "conspiracy", "cold case"], roles: ["a teenage detective", "a sharp-eyed librarian"], settings: ["a mansion cut off by snow", "a city of whispered rumors"], goals: ["to solve a locked room murder", "to expose a decades old conspiracy"] },
    GenreInfo { name: "Horror", tags: ["curse", "survival", "haunted town", "monsters"], roles: ["a cursed exchange student", "a lone night guard"], settings: ["a haunted mountain town", "an abandoned hospital"], goals: ["to break a deadly curse", "to survive until dawn"] },
    GenreInfo { name: "Sports", tags: ["volleyball", "basketball", "underdog team", "nationals"], roles: ["an undersized ace", "a stubborn team captain"], settings: ["a forgotten high school gym", "the national championship court"], goals: ["to reach the nationals", "to rebuild a fallen team"] },
    GenreInfo { name: "Mecha", tags: ["giant robots", "pilot academy", "war", "super prototype"], roles: ["a reluctant mech pilot", "a prototype engineer"], settings: ["a besieged orbital fortress", "a frontline pilot academy"], goals: ["to pilot the last super prototype", "to end an endless war"] },
    GenreInfo { name: "Isekai", tags: ["reincarnation", "game world", "overpowered hero", "dark lord"], roles: ["an office worker reborn as a slime", "a gamer trapped in a game"], settings: ["a world of game menus", "a dark lord's realm"], goals: ["to defeat the dark lord", "to log out alive"] },
    GenreInfo { name: "Psychological", tags: ["mind games", "unreliable narrator", "death game", "moral dilemma"], roles: ["a genius manipulator", "a contestant with no memory"], settings: ["a deadly game show", "a sealed experimental facility"], goals: ["to outwit every opponent", "to uncover who they really are"] },
    GenreInfo { name: "Supernatural", tags: ["yokai", "exorcist", "shrine", "hidden powers"], roles: ["a shrine maiden", "a boy who sees yokai"], settings: ["a shrine between worlds", "a town full of yokai"], goals: ["to seal a rampaging yokai", "to control hidden powers"] },
    GenreInfo { name: "Music", tags: ["idol group", "band", "orchestra", "concert tour"], roles: ["a shy guitarist", "an idol trainee"], settings: ["a cramped live house", "a famous concert hall"], goals: ["to play the big festival stage", "to debut as an idol group"] },
];

pub const CLUSTERS: [(&str, [&str; 3]); 6] = [
    ("Battle Shonen", ["Action", "Adventure", "Supernatural"]),
    ("Mecha and Space", ["Sci-Fi", "Mecha", "Drama"]),
    ("Cozy Hearts", ["Slice of Life", "Romance", "Comedy"]),
    ("Dark Mysteries", ["Mystery", "Horror", "Psychological"]),
    ("Court and Stage", ["Sports", "Music", "Drama"]),
    ("Other Worlds", ["Isekai", "Fantasy", "Adventure"]),
];

const PREFIXES: [&str; 24] = [
    "Crimson", "Silent", "Neon", "Eternal", "Hollow", "Starlit", "Iron", "Velvet", "Phantom", "Azure", "Golden", "Midnight",
    "Paper", "Shattered", "Infinite", "Wandering", "Frozen", "Burning", "Lunar", "Hidden", "Electric", "Last", "Whispering", "Sakura",
];

const NOUNS: [&str; 20] = [
    "Horizon", "Ronin", "Requiem", "Blossom", "Circuit", "Academy", "Odyssey", "Crown", "Garden", "Melody",
    "Tides", "Covenant", "Frontier", "Lantern", "Parade", "Signal", "Kingdom", "Spiral", "Express", "Archive",
];

const FIRST_NAMES: [&str; 20] = [
    "Aiko", "Haru", "Ren", "Yuki", "Sora", "Kaito", "Mei", "Riku", "Hana", "Daichi",
    "Emi", "Takumi", "Nao", "Kenji", "Rin", "Shun", "Miyu", "Itsuki", "Akari", "Yuto",
];

pub struct Dataset {
    pub titles: Vec<NewTitle>,
    pub users: Vec<NewUser>,
    pub watches: Vec<(usize, usize)>,
}

pub fn genre(name: &str) -> Option<&'static GenreInfo> {
    GENRES.iter().find(|g| g.name == name)
}

pub fn cluster_genres(name: &str) -> Vec<String> {
    CLUSTERS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, gs)| gs.iter().map(|g| g.to_string()).collect())
        .unwrap_or_default()
}

pub fn generate() -> Dataset {
    let mut rng = Rng::new(SEED);
    let titles = generate_titles(&mut rng);
    let popularity = popularity_weights(&mut rng, titles.len());
    let (users, secondary) = generate_users(&mut rng);
    let watches = generate_watches(&mut rng, &titles, &users, &secondary, &popularity);
    Dataset { titles, users, watches }
}

pub struct Indexed {
    pub titles: Vec<Title>,
    pub histories: HashMap<i32, Vec<i32>>,
    pub user_genres: HashMap<i32, Vec<String>>,
}

pub fn indexed() -> Indexed {
    let d = generate();
    let titles = d
        .titles
        .into_iter()
        .enumerate()
        .map(|(i, t)| Title { id: i as i32 + 1, name: t.name, genres: t.genres, tags: t.tags, synopsis: t.synopsis, year: t.year, is_new: false })
        .collect();
    let mut histories: HashMap<i32, Vec<i32>> = HashMap::new();
    for (u, t) in d.watches {
        histories.entry(u as i32 + 1).or_default().push(t as i32 + 1);
    }
    let user_genres = d
        .users
        .iter()
        .enumerate()
        .map(|(i, u)| (i as i32 + 1, cluster_genres(u.cluster.as_deref().unwrap_or(""))))
        .collect();
    Indexed { titles, histories, user_genres }
}

fn generate_titles(rng: &mut Rng) -> Vec<NewTitle> {
    let mut names = HashSet::new();
    let mut titles = Vec::with_capacity(TITLE_COUNT);
    while titles.len() < TITLE_COUNT {
        let name = format!("{} {}", PREFIXES[rng.below(PREFIXES.len())], NOUNS[rng.below(NOUNS.len())]);
        if !names.insert(name.clone()) {
            continue;
        }
        let cluster = &CLUSTERS[titles.len() % CLUSTERS.len()].1;
        let first = rng.below(3);
        let second = (first + 1 + rng.below(2)) % 3;
        let mut genres = vec![cluster[first].to_string(), cluster[second].to_string()];
        if rng.chance(0.3) {
            let extra = GENRES[rng.below(GENRES.len())].name.to_string();
            if !genres.contains(&extra) {
                genres.push(extra);
            }
        }
        titles.push(build_title(rng, name, genres));
    }
    titles
}

fn build_title(rng: &mut Rng, name: String, genres: Vec<String>) -> NewTitle {
    let infos: Vec<&GenreInfo> = genres.iter().filter_map(|g| genre(g)).collect();
    let mut tags = Vec::new();
    for info in &infos {
        for _ in 0..2 {
            let tag = info.tags[rng.below(4)].to_string();
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
    }
    let lead = infos[0];
    let other = infos[1 % infos.len()];
    let synopsis = format!(
        "In {}, {} fights {}. Along the way they meet {} hoping {}.",
        lead.settings[rng.below(2)],
        lead.roles[rng.below(2)],
        lead.goals[rng.below(2)],
        other.roles[rng.below(2)],
        other.goals[rng.below(2)],
    );
    NewTitle { name, genres, tags, synopsis, year: 1998 + rng.below(28) as i32, is_new: false }
}

fn popularity_weights(rng: &mut Rng, n: usize) -> Vec<f64> {
    let mut ranks: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = rng.below(i + 1);
        ranks.swap(i, j);
    }
    ranks.iter().map(|r| 1.0 / ((*r + 1) as f64).powf(0.8)).collect()
}

fn generate_users(rng: &mut Rng) -> (Vec<NewUser>, Vec<Option<usize>>) {
    (0..USER_COUNT)
        .map(|i| {
            let secondary = if rng.chance(0.3) { Some(rng.below(CLUSTERS.len())) } else { None };
            let user = NewUser {
                name: format!("{} {:03}", FIRST_NAMES[rng.below(FIRST_NAMES.len())], i + 1),
                cluster: Some(CLUSTERS[i % CLUSTERS.len()].0.to_string()),
                picked_genres: Vec::new(),
            };
            (user, secondary)
        })
        .unzip()
}

fn affinity(title: &NewTitle, cluster: &[&str; 3]) -> f64 {
    title.genres.iter().filter(|g| cluster.contains(&g.as_str())).count() as f64
}

fn generate_watches(
    rng: &mut Rng,
    titles: &[NewTitle],
    users: &[NewUser],
    secondaries: &[Option<usize>],
    popularity: &[f64],
) -> Vec<(usize, usize)> {
    let mut watches = Vec::new();
    for (u, user) in users.iter().enumerate() {
        let primary = CLUSTERS.iter().find(|(n, _)| user.cluster.as_deref() == Some(*n)).map(|c| &c.1);
        let secondary = secondaries[u].map(|s| &CLUSTERS[s].1);
        let count = 12 + rng.below(25);
        let mut seen = HashSet::new();
        while seen.len() < count {
            let roll = rng.unit();
            let cluster = match (primary, secondary) {
                (Some(p), _) if roll < 0.6 => Some(p),
                (Some(_), Some(s)) if roll < 0.85 => Some(s),
                (Some(p), None) if roll < 0.85 => Some(p),
                _ => None,
            };
            let weights: Vec<f64> = titles
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    if seen.contains(&i) {
                        return 0.0;
                    }
                    match cluster {
                        Some(c) => popularity[i] * affinity(t, c),
                        None => popularity[i],
                    }
                })
                .collect();
            match rng.weighted(&weights) {
                Some(i) => {
                    seen.insert(i);
                    watches.push((u, i));
                }
                None => break,
            }
        }
    }
    watches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_matches_the_spec_sizes_and_is_reproducible() {
        let a = generate();
        let b = generate();
        assert_eq!(a.titles.len(), TITLE_COUNT);
        assert_eq!(a.users.len(), USER_COUNT);
        assert_eq!(a.watches, b.watches);
        let names: HashSet<&str> = a.titles.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names.len(), TITLE_COUNT);
    }

    #[test]
    fn users_mostly_watch_inside_their_taste_cluster() {
        let d = generate();
        let mut inside = 0usize;
        for (u, t) in &d.watches {
            let cluster = cluster_genres(d.users[*u].cluster.as_deref().unwrap_or(""));
            if d.titles[*t].genres.iter().any(|g| cluster.contains(g)) {
                inside += 1;
            }
        }
        let ratio = inside as f64 / d.watches.len() as f64;
        assert!(ratio > 0.6, "taste clusters too weak: {ratio}");
    }
}
