use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use dioxus::prelude::*;
use image::ImageEncoder;
use pubky::{Capabilities, Pubky, PubkyAuthFlow, PubkySession, PublicStorage};
use uuid::Uuid;

use crate::utils::extract_title;

mod create_wiki;
mod edit_wiki;
mod utils;
mod view_wiki;

const APP_NAME: &str = "Pubky Wiki";

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[derive(Clone, Debug)]
pub(crate) enum AuthState {
    Initializing,
    ShowingQR {
        auth_url: String,
    },
    Authenticated {
        session: PubkySession,
        pub_storage: PublicStorage,
        file_cache: HashMap<String, String>,
    },
    Error(String),
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum ViewState {
    WikiList,
    CreateWiki,
    ViewWiki,
    EditWiki,
}

fn App() -> Element {
    let mut auth_state = use_signal(|| AuthState::Initializing);
    let mut view_state = use_signal(|| ViewState::WikiList);
    let mut edit_wiki_content = use_signal(String::new);
    let mut selected_wiki_page_id = use_signal(String::new);
    let mut selected_wiki_content = use_signal(String::new);
    let mut selected_wiki_user_id = use_signal(String::new);
    let mut selected_wiki_fork_urls = use_signal(|| Vec::<String>::new());
    let mut show_copy_tooltip = use_signal(|| false);
    let mut forked_from_page_id = use_signal(|| Option::<String>::None);

    // Initialize authentication on first render
    use_effect(move || {
        spawn(async move {
            match initialize_auth().await {
                Ok((pubky, flow, auth_url)) => {
                    auth_state.set(AuthState::ShowingQR {
                        auth_url: auth_url.clone(),
                    });

                    // Poll for authentication
                    match flow.await_approval().await {
                        Ok(session) => {
                            let pub_storage = pubky.public_storage();
                            let file_cache = fetch_file_cache(&session, &pub_storage).await;

                            auth_state.set(AuthState::Authenticated {
                                session,
                                pub_storage,
                                file_cache,
                            });
                        }
                        Err(e) => {
                            auth_state.set(AuthState::Error(format!("Authentication failed: {e}")));
                        }
                    }
                }
                Err(e) => {
                    auth_state.set(AuthState::Error(format!("Failed to initialize: {e}")));
                }
            }
        });
    });

    rsx! {
        style { {include_str!("../assets/main.css")} }
        div {
            class: "container",
            div {
                class: "header",
                div {
                    class: "logo",
                    "📚"
                }
                h1 { "{APP_NAME}" }
            }

            match auth_state() {
                AuthState::Initializing => rsx! {
                    div { class: "center-content",
                        div { class: "spinner" }
                        p { "Initializing authentication..." }
                    }
                },
                AuthState::ShowingQR { auth_url } => rsx! {
                    div { class: "center-content",
                        p { "Scan this QR code with your Pubky app to login:" }
                        div { class: "qr-container",
                            img {
                                class: "qr-code",
                                src: "{generate_qr_data_url(&auth_url)}",
                                alt: "QR Code"
                            }
                        }
                        p { class: "waiting-text", "Waiting for authentication..." }
                        div { class: "spinner" }
                    }
                },
                AuthState::Authenticated { session, pub_storage, file_cache } => {
                    match view_state() {
                        ViewState::WikiList => {
                            WikiList(
                                session,
                                file_cache,
                                view_state,
                                selected_wiki_page_id,
                                selected_wiki_user_id,
                                selected_wiki_content,
                                selected_wiki_fork_urls,
                                pub_storage,
                            )
                        },
                        ViewState::CreateWiki => {
                            create_wiki::CreateWiki(
                                session,
                                view_state,
                                edit_wiki_content,
                                forked_from_page_id,
                                auth_state,
                            )
                        },
                        ViewState::EditWiki => {
                            edit_wiki::EditWiki(
                                session,
                                view_state,
                                edit_wiki_content,
                                selected_wiki_page_id,
                                selected_wiki_content,
                                auth_state,
                            )
                        },
                        ViewState::ViewWiki => {
                            view_wiki::ViewWiki(
                                session,
                                pub_storage,
                                view_state,
                                selected_wiki_page_id,
                                selected_wiki_content,
                                selected_wiki_user_id,
                                selected_wiki_fork_urls,
                                edit_wiki_content,
                                forked_from_page_id,
                                show_copy_tooltip,
                            )
                        },
                    }
                },
                AuthState::Error(error) => rsx! {
                    div { class: "error-content",
                        p { class: "error-title", "Error" }
                        p { "{error}" }
                    }
                },
            }
        }
    }
}

fn WikiList(
    session: PubkySession,
    file_cache: HashMap<String, String>,
    mut view_state: Signal<ViewState>,
    mut selected_wiki_page_id: Signal<String>,
    mut selected_wiki_user_id: Signal<String>,
    mut selected_wiki_content: Signal<String>,
    mut selected_wiki_fork_urls: Signal<Vec<String>>,
    pub_storage: PublicStorage,
) -> Element {
    rsx! {
        div { class: "wiki-list",
            button {
                class: "btn btn-primary",
                onclick: move |_| view_state.set(ViewState::CreateWiki),
                "✨ Create New Wiki Page"
            }

            h2 { "My Wiki Posts" }

            if file_cache.is_empty() {
                p { class: "empty-state", "No wiki posts yet. Create your first one!" }
            } else {
                div { class: "wiki-items",
                    for (file_url, file_title) in file_cache {
                        {
                            let file_name = file_url.split('/').last().unwrap_or(&file_url).to_string();
                            let pk = session.info().public_key().to_string();
                            let session_clone = session.clone();
                            let pub_storage_clone = pub_storage.clone();
                            
                            rsx! {
                                div { class: "wiki-item",
                                    button {
                                        class: "wiki-item-id",
                                        onclick: move |_| {
                                            selected_wiki_user_id.set(pk.clone());
                                            selected_wiki_page_id.set(file_name.clone());
                                            selected_wiki_content.set(String::new());
                                            
                                            let session_c = session_clone.clone();
                                            let page_id = file_name.clone();
                                            let pub_storage_c = pub_storage_clone.clone();
                                            spawn(async move {
                                                let fork_urls = discover_fork_urls(&session_c, &pub_storage_c, &page_id).await;
                                                selected_wiki_fork_urls.set(fork_urls);
                                            });
                                            
                                            view_state.set(ViewState::ViewWiki);
                                        },
                                        "{file_name}"
                                    }
                                    span { class: "wiki-item-title", "{file_title}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn fetch_file_cache(session: &PubkySession, pub_storage: &PublicStorage) -> HashMap<String, String> {
    let mut file_cache = HashMap::new();

    match get_list_async(session, "/pub/wiki.app/").await {
        Ok(file_urls) => {
            for file_url in &file_urls {
                match pub_storage.get(file_url).await {
                    Ok(response) => {
                        match response.text().await {
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

    file_cache
}

async fn discover_fork_urls(
    session: &PubkySession,
    pub_storage: &PublicStorage,
    page_id: &str,
) -> Vec<String> {
    let follows = get_my_follows_async(session).await;
    let mut result = vec![];

    let own_pk = session.info().public_key().to_string();
    result.push(format!("{own_pk}/{page_id}"));

    for follow_pk in follows {
        let fork_path = format!("pubky://{follow_pk}/pub/wiki.app/{page_id}");
        log::info!("fork_path = {fork_path}");

        match pub_storage.get(&fork_path).await {
            Ok(_) => result.push(format!("{follow_pk}/{page_id}")),
            Err(e) => log::error!("Failed to check if file exists: {e}"),
        }
    }
    result
}

async fn get_my_follows_async(session: &PubkySession) -> Vec<String> {
    get_list_async(session, "/pub/pubky.app/follows/")
        .await
        .map(|list| {
            list.iter()
                .map(|path| path.split('/').last().unwrap_or(&path).to_string())
                .collect()
        })
        .unwrap_or_default()
}

async fn get_list_async(session: &PubkySession, folder_path: &str) -> Result<Vec<String>> {
    let session_storage = session.storage();
    let list_req = session_storage
        .list(folder_path)
        .map_err(|e| anyhow!("Failed to create list request: {e}"))?;

    log::info!("listing {folder_path}");

    let entries = list_req.send().await?;
    let result_list = entries.iter().map(|entry| entry.to_pubky_url()).collect();

    Ok(result_list)
}

fn generate_qr_data_url(url: &str) -> String {
    match qrcode::QrCode::new(url.as_bytes()) {
        Ok(qr) => {
            let qr_image = qr.render::<image::Luma<u8>>().build();
            
            // Convert to PNG bytes
            let mut png_bytes = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
            let (width, height) = qr_image.dimensions();
            if encoder.write_image(&qr_image, width, height, image::ExtendedColorType::L8).is_ok() {
                let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                return format!("data:image/png;base64,{}", base64_data);
            }
        }
        Err(e) => log::error!("Failed to generate QR code: {e}"),
    }
    String::new()
}

async fn initialize_auth() -> Result<(Pubky, PubkyAuthFlow, String)> {
    let pubky = Pubky::new()?;
    let caps = Capabilities::builder().write("/pub/wiki.app/").finish();
    let flow = pubky.start_auth_flow(&caps)?;
    let auth_url = flow.authorization_url().to_string();

    Ok((pubky, flow, auth_url))
}

pub(crate) async fn create_wiki_post(
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

pub(crate) async fn update_wiki_post(
    session: &PubkySession,
    page_id: &str,
    content: &str,
) -> Result<()> {
    let path = format!("/pub/wiki.app/{}", page_id);

    session.storage().put(&path, content.to_string()).await?;

    log::info!("Updated post at path: {}", path);

    Ok(())
}

pub(crate) async fn delete_wiki_post(session: &PubkySession, page_id: &str) -> Result<()> {
    let path = format!("/pub/wiki.app/{}", page_id);

    session.storage().delete(&path).await?;

    log::info!("Deleted post at path: {}", path);

    Ok(())
}
