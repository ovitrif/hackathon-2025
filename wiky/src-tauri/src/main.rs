// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use pubky_wiki_lib::{
    initialize_auth, create_wiki_post, update_wiki_post, delete_wiki_post,
    AppState, AuthState, utils::{generate_qr_image_base64, get_list}
};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, State};

#[derive(Clone, Serialize, Deserialize)]
struct WikiPage {
    id: String,
    title: String,
    url: String,
}

#[tauri::command]
fn get_auth_state(state: State<'_, AppState>) -> Result<AuthState, String> {
    let auth_state = state.auth_state.lock().unwrap().clone();
    Ok(auth_state)
}

#[tauri::command]
async fn start_authentication(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let state_clone = Arc::new((*state).clone());

    tokio::spawn(async move {
        match initialize_auth().await {
            Ok((pubky, flow, auth_url)) => {
                // Update state with QR code
                {
                    let mut auth_state = state_clone.auth_state.lock().unwrap();
                    *auth_state = AuthState::ShowingQR {
                        auth_url: auth_url.clone(),
                    };
                }

                // Emit event to frontend
                let _ = app_handle.emit("auth-state-changed", ());

                // Wait for authentication
                match flow.await_approval().await {
                    Ok(session) => {
                        let pub_storage = pubky.public_storage();

                        // Store session and pub_storage
                        {
                            let mut session_lock = state_clone.session.lock().unwrap();
                            *session_lock = Some(session.clone());

                            let mut pub_storage_lock = state_clone.pub_storage.lock().unwrap();
                            *pub_storage_lock = Some(pub_storage.clone());
                        }

                        // Fetch files and update state
                        state_clone.fetch_files_and_update(&session, &pub_storage).await;

                        // Emit event to frontend
                        let _ = app_handle.emit("auth-state-changed", ());
                    }
                    Err(e) => {
                        let mut auth_state = state_clone.auth_state.lock().unwrap();
                        *auth_state = AuthState::Error {
                            message: format!("Authentication failed: {e}"),
                        };
                        let _ = app_handle.emit("auth-state-changed", ());
                    }
                }
            }
            Err(e) => {
                let mut auth_state = state_clone.auth_state.lock().unwrap();
                *auth_state = AuthState::Error {
                    message: format!("Failed to initialize: {e}"),
                };
                let _ = app_handle.emit("auth-state-changed", ());
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn get_wiki_pages(state: State<'_, AppState>) -> Result<Vec<WikiPage>, String> {
    let auth_state = state.auth_state.lock().unwrap().clone();

    match auth_state {
        AuthState::Authenticated { file_cache, .. } => {
            let pages: Vec<WikiPage> = file_cache
                .iter()
                .map(|(url, title)| {
                    let id = url.split('/').last().unwrap_or(url).to_string();
                    WikiPage {
                        id,
                        title: title.clone(),
                        url: url.clone(),
                    }
                })
                .collect();
            Ok(pages)
        }
        _ => Err("Not authenticated".to_string()),
    }
}

#[tauri::command]
async fn get_wiki_content(
    state: State<'_, AppState>,
    user_id: String,
    page_id: String,
) -> Result<String, String> {
    let pub_storage = {
        let pub_storage_lock = state.pub_storage.lock().unwrap();
        pub_storage_lock
            .as_ref()
            .ok_or("Not authenticated")?
            .clone()
    };

    let path = format!("pubky://{user_id}/pub/wiki.app/{page_id}");

    let get_path_fut = pub_storage.get(&path);
    match get_path_fut.await {
        Ok(response) => match response.text().await {
            Ok(text) => Ok(text),
            Err(e) => Err(format!("Error reading content: {e}")),
        },
        Err(e) => Err(format!("Error fetching path {path}: {e}")),
    }
}

#[tauri::command]
async fn create_wiki(
    state: State<'_, AppState>,
    content: String,
    filename: Option<String>,
) -> Result<String, String> {
    let session = {
        let session_lock = state.session.lock().unwrap();
        session_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    let filename_ref = filename.as_deref();
    match create_wiki_post(&session, &content, filename_ref).await {
        Ok(path) => {
            // Refresh file cache
            let pub_storage = {
                let pub_storage_lock = state.pub_storage.lock().unwrap();
                pub_storage_lock.as_ref().unwrap().clone()
            };

            state.fetch_files_and_update(&session, &pub_storage).await;
            Ok(path)
        }
        Err(e) => Err(format!("Failed to create wiki: {e}")),
    }
}

#[tauri::command]
async fn update_wiki(
    state: State<'_, AppState>,
    page_id: String,
    content: String,
) -> Result<(), String> {
    let session = {
        let session_lock = state.session.lock().unwrap();
        session_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    match update_wiki_post(&session, &page_id, &content).await {
        Ok(_) => {
            // Refresh file cache
            let pub_storage = {
                let pub_storage_lock = state.pub_storage.lock().unwrap();
                pub_storage_lock.as_ref().unwrap().clone()
            };

            state.fetch_files_and_update(&session, &pub_storage).await;
            Ok(())
        }
        Err(e) => Err(format!("Failed to update wiki: {e}")),
    }
}

#[tauri::command]
async fn delete_wiki(state: State<'_, AppState>, page_id: String) -> Result<(), String> {
    let session = {
        let session_lock = state.session.lock().unwrap();
        session_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    match delete_wiki_post(&session, &page_id).await {
        Ok(_) => {
            // Refresh file cache
            let pub_storage = {
                let pub_storage_lock = state.pub_storage.lock().unwrap();
                pub_storage_lock.as_ref().unwrap().clone()
            };

            state.fetch_files_and_update(&session, &pub_storage).await;
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete wiki: {e}")),
    }
}

#[tauri::command]
fn get_qr_image(state: State<'_, AppState>) -> Result<String, String> {
    let auth_state = state.auth_state.lock().unwrap().clone();

    if let AuthState::ShowingQR { auth_url } = auth_state {
        generate_qr_image_base64(&auth_url)
            .ok_or_else(|| "Failed to generate QR code".to_string())
    } else {
        Err("Not in QR state".to_string())
    }
}

#[tauri::command]
async fn get_follows(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let session = {
        let session_lock = state.session.lock().unwrap();
        session_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    match get_list(&session, "/pub/pubky.app/follows/").await {
        Ok(list) => {
            let follows: Vec<String> = list
                .iter()
                .map(|path| {
                    path.split('/')
                        .last()
                        .unwrap_or(path)
                        .to_string()
                })
                .collect();
            Ok(follows)
        }
        Err(e) => Err(format!("Failed to get follows: {e}")),
    }
}

#[tauri::command]
async fn discover_forks(
    state: State<'_, AppState>,
    page_id: String,
) -> Result<Vec<String>, String> {
    let session = {
        let session_lock = state.session.lock().unwrap();
        session_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    let pub_storage = {
        let pub_storage_lock = state.pub_storage.lock().unwrap();
        pub_storage_lock.as_ref().ok_or("Not authenticated")?.clone()
    };

    let follows = match get_list(&session, "/pub/pubky.app/follows/").await {
        Ok(list) => list
            .iter()
            .map(|path| path.split('/').last().unwrap_or(path).to_string())
            .collect::<Vec<String>>(),
        Err(_) => vec![],
    };

    let mut result = vec![];

    // Add the current user's version as a fork (root version)
    let own_pk = session.info().public_key().to_string();
    result.push(format!("{own_pk}/{page_id}"));

    for follow_pk in follows {
        let fork_path = format!("pubky://{follow_pk}/pub/wiki.app/{page_id}");
        let exists_fut = pub_storage.get(&fork_path);

        match exists_fut.await {
            Ok(_) => result.push(format!("{follow_pk}/{page_id}")),
            Err(_) => {}
        }
    }

    Ok(result)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new().expect("Failed to create app state");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_auth_state,
            start_authentication,
            get_wiki_pages,
            get_wiki_content,
            create_wiki,
            update_wiki,
            delete_wiki,
            get_qr_image,
            get_follows,
            discover_forks,
        ])
        .setup(|app| {
            let state = app.state::<AppState>();
            let app_handle = app.handle().clone();

            // Start authentication automatically
            let state_clone = Arc::new((*state).clone());
            tauri::async_runtime::spawn(async move {
                match initialize_auth().await {
                    Ok((pubky, flow, auth_url)) => {
                        // Update state with QR code
                        {
                            let mut auth_state = state_clone.auth_state.lock().unwrap();
                            *auth_state = AuthState::ShowingQR {
                                auth_url: auth_url.clone(),
                            };
                        }

                        // Emit event to frontend
                        let _ = app_handle.emit("auth-state-changed", ());

                        // Wait for authentication
                        match flow.await_approval().await {
                            Ok(session) => {
                                let pub_storage = pubky.public_storage();

                                // Store session and pub_storage
                                {
                                    let mut session_lock = state_clone.session.lock().unwrap();
                                    *session_lock = Some(session.clone());

                                    let mut pub_storage_lock = state_clone.pub_storage.lock().unwrap();
                                    *pub_storage_lock = Some(pub_storage.clone());
                                }

                                // Fetch files and update state
                                state_clone.fetch_files_and_update(&session, &pub_storage).await;

                                // Emit event to frontend
                                let _ = app_handle.emit("auth-state-changed", ());
                            }
                            Err(e) => {
                                let mut auth_state = state_clone.auth_state.lock().unwrap();
                                *auth_state = AuthState::Error {
                                    message: format!("Authentication failed: {e}"),
                                };
                                let _ = app_handle.emit("auth-state-changed", ());
                            }
                        }
                    }
                    Err(e) => {
                        let mut auth_state = state_clone.auth_state.lock().unwrap();
                        *auth_state = AuthState::Error {
                            message: format!("Failed to initialize: {e}"),
                        };
                        let _ = app_handle.emit("auth-state-changed", ());
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
