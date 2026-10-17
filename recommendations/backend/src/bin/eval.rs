use recs::eval::{evaluate, markdown};
use recs::seed;

fn main() {
    let d = seed::indexed();
    let watches: usize = d.histories.values().map(|h| h.len()).sum();
    println!("titles {} users {} watches {}", d.titles.len(), d.histories.len(), watches);
    print!("{}", markdown(&evaluate(&d.titles, &d.histories, &d.user_genres)));
}
