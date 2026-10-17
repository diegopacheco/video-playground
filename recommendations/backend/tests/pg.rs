use recs::embed::cosine;
use recs::service::Service;
use recs::store::Store;
use recs::store::pg::PgStore;
use tokio::sync::Mutex;

static DB: Mutex<()> = Mutex::const_new(());

fn url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "postgres://recs:recs@localhost:6115/recs_test".into())
}

async fn fresh() -> PgStore {
    let store = PgStore::connect(&url()).await.expect("postgres must be running, start it with ./scripts/start-all.sh");
    store.reset().await.expect("reset");
    store
}

#[tokio::test]
async fn pgvector_cosine_distance_agrees_with_rust_embeddings() {
    let _guard = DB.lock().await;
    let service = Service::bootstrap(fresh().await).await.expect("bootstrap");
    assert_eq!(service.titles().expect("titles").len(), 100);
    assert_eq!(service.users().expect("users").len(), 500);

    let detail = service.title(1).await.expect("title");
    assert_eq!(detail.similar_content.len(), 10);
    assert!(detail.similar_content.iter().all(|i| i.title.id != 1));
    let scores: Vec<f32> = detail.similar_content.iter().map(|i| i.score).collect();
    assert!(scores.windows(2).all(|w| w[0] >= w[1]), "pgvector must return nearest first: {scores:?}");

    let store = PgStore::connect(&url()).await.expect("connect");
    let probe = recs::seed::indexed();
    let model = recs::model::Model::build(&probe.titles, &probe.histories);
    let query = model.embedding(1).expect("embedding");
    let hits = store.nearest(query, 3).await.expect("nearest");
    assert_eq!(hits[0].0, 1, "a title is its own nearest neighbor");
    for (id, distance) in hits {
        let expected = 1.0 - cosine(query, model.embedding(id).expect("embedding"));
        assert!((distance - expected).abs() < 1e-3, "id {id}: pg {distance} vs rust {expected}");
    }
}

#[tokio::test]
async fn data_survives_a_restart_without_reseeding() {
    let _guard = DB.lock().await;
    let first = Service::bootstrap(fresh().await).await.expect("bootstrap");
    let user = first.create_user("Persisted".into(), vec!["Music".into()]).await.expect("user");
    first.add_watch(user.id, 3).await.expect("watch");

    let again = Service::bootstrap(PgStore::connect(&url()).await.expect("connect")).await.expect("bootstrap");
    assert_eq!(again.titles().expect("titles").len(), 100);
    let detail = again.user(user.id).expect("user");
    assert_eq!(detail.user.picked_genres, vec!["Music".to_string()]);
    assert_eq!(detail.watched.iter().map(|t| t.id).collect::<Vec<_>>(), vec![3]);
}
