use std::collections::HashMap;
use std::sync::Arc;

use drm::audit::AuditLog;
use drm::config::Settings;
use drm::crypto::hex_decode;
use drm::http::{AppState, router};
use drm::keystore::MemoryKeyStore;
use drm::license::{ClearKeyService, LicenseService};
use drm::session::SessionSigner;
use drm::url_signer::UrlSigner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::from_env()?;
    let store = Arc::new(MemoryKeyStore::from_hex(&settings.drm.key_entries())?);
    let mut licenses: HashMap<String, Arc<dyn LicenseService>> = HashMap::new();
    licenses.insert("clearkey".into(), Arc::new(ClearKeyService::new(store)));
    let state = AppState {
        sessions: SessionSigner::new(hex_decode(&settings.drm.session_secret)?),
        urls: UrlSigner::new(hex_decode(&settings.drm.url_secret)?),
        licenses,
        media_dir: settings.media_dir,
        audit: AuditLog::new(50),
    };
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", settings.port)).await?;
    println!("drm server on http://localhost:{}", settings.port);
    axum::serve(listener, router(state)).await?;
    Ok(())
}
