use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use pubky::{Capabilities, Pubky, PubkyAuthFlow, PubkySession, PublicStorage};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub mod utils;

use utils::{extract_title, get_list};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthState {
    Initializing,
    ShowingQR { auth_url: String },
    Authenticated {
        public_key: String,
        file_cache: HashMap<String, String>,
    },
    Error { message: String },
}

#[derive(Clone)]
pub struct AppState {
    pub auth_state: Arc<Mutex<AuthState>>,
    pub session: Arc<Mutex<Option<PubkySession>>>,
    pub pub_storage: Arc<Mutex<Option<PublicStorage>>>,
    pub rt: Arc<Runtime>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let rt = Runtime::new()?;
        Ok(Self {
            auth_state: Arc::new(Mutex::new(AuthState::Initializing)),
            session: Arc::new(Mutex::new(None)),
            pub_storage: Arc::new(Mutex::new(None)),
            rt: Arc::new(rt),
        })
    }

    pub fn fetch_files_and_update(&self, session: &PubkySession, pub_storage: &PublicStorage) {
        let mut file_cache = HashMap::new();

        match get_list(session, "/pub/wiki.app/", self.rt.clone()) {
            Ok(file_urls) => {
                for file_url in &file_urls {
                    let get_path_fut = pub_storage.get(file_url);
                    match self.rt.block_on(get_path_fut) {
                        Ok(response) => {
                            let response_text_fut = response.text();
                            match self.rt.block_on(response_text_fut) {
                                Ok(content) => {
                                    let file_title = extract_title(&content);
                                    file_cache.insert(file_url.clone(), file_title.to_string());
                                }
                                Err(e) => log::error!("Error reading content: {e}"),
                            }
                        }
                        Err(e) => log::error!("Error fetching path {file_url}: {e}"),
                    }
                }
            }
            Err(e) => log::error!("Failed to list files: {e}"),
        }

        let public_key = session.info().public_key().to_string();
        *self.auth_state.lock().unwrap() = AuthState::Authenticated {
            public_key,
            file_cache,
        };
    }
}

pub async fn initialize_auth() -> Result<(Pubky, PubkyAuthFlow, String)> {
    let pubky = Pubky::new()?;
    let caps = Capabilities::builder().write("/pub/wiki.app/").finish();
    let flow = pubky.start_auth_flow(&caps)?;
    let auth_url = flow.authorization_url().to_string();

    Ok((pubky, flow, auth_url))
}

pub async fn create_wiki_post(
    session: &PubkySession,
    content: &str,
    filename: Option<&str>,
) -> Result<String> {
    let path = if let Some(fname) = filename {
        format!("/pub/wiki.app/{}", fname)
    } else {
        format!("/pub/wiki.app/{}", Uuid::new_v4())
    };

    session.storage().put(&path, content.to_string()).await?;
    log::info!("Created post at path: {}", path);

    Ok(path)
}

pub async fn update_wiki_post(
    session: &PubkySession,
    page_id: &str,
    content: &str,
) -> Result<()> {
    let path = format!("/pub/wiki.app/{}", page_id);
    session.storage().put(&path, content.to_string()).await?;
    log::info!("Updated post at path: {}", path);

    Ok(())
}

pub async fn delete_wiki_post(session: &PubkySession, page_id: &str) -> Result<()> {
    let path = format!("/pub/wiki.app/{}", page_id);
    session.storage().delete(&path).await?;
    log::info!("Deleted post at path: {}", path);

    Ok(())
}
