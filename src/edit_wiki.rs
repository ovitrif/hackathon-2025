use crate::{delete_wiki_post, update_wiki_post, AuthState, ViewState};
use dioxus::prelude::*;
use pubky::PubkySession;

pub fn EditWiki(
    session: PubkySession,
    mut view_state: Signal<ViewState>,
    mut edit_wiki_content: Signal<String>,
    mut selected_wiki_page_id: Signal<String>,
    mut selected_wiki_content: Signal<String>,
    auth_state: Signal<AuthState>,
) -> Element {
    let session_update = session.clone();
    let session_delete = session.clone();
    
    rsx! {
        div { class: "edit-wiki",
            h2 { "Edit Wiki Page" }

            label { "Content:" }
            textarea {
                class: "wiki-editor",
                value: "{edit_wiki_content}",
                rows: "15",
                oninput: move |evt| edit_wiki_content.set(evt.value())
            }

            div { class: "button-group",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let session_clone = session_update.clone();
                        let content = edit_wiki_content();
                        let page_id = selected_wiki_page_id();
                        let content_clone = content.clone();
                        
                        spawn(async move {
                            match update_wiki_post(&session_clone, &page_id, &content).await {
                                Ok(_) => {
                                    log::info!("Updated wiki post: {}", page_id);
                                }
                                Err(e) => log::error!("Failed to update wiki post: {e}"),
                            }
                        });

                        selected_wiki_content.set(content_clone);
                        edit_wiki_content.set(String::new());
                        view_state.set(ViewState::WikiList);
                    },
                    "✓ Update"
                }

                button {
                    class: "btn btn-danger",
                    onclick: move |_| {
                        let session_clone = session_delete.clone();
                        let page_id = selected_wiki_page_id();
                        let mut auth_state_clone = auth_state;
                        
                        spawn(async move {
                            match delete_wiki_post(&session_clone, &page_id).await {
                                Ok(_) => {
                                    log::info!("Deleted wiki post: {}", page_id);

                                    // Remove from file cache
                                    if let AuthState::Authenticated { session, ref mut file_cache, .. } = &mut *auth_state_clone.write() {
                                        let own_user_pk = session.info().public_key().to_string();
                                        let file_url = format!("pubky://{own_user_pk}/pub/wiki.app/{page_id}");
                                        file_cache.remove(&file_url);
                                    }
                                }
                                Err(e) => log::error!("Failed to delete wiki post: {e}"),
                            }
                        });

                        edit_wiki_content.set(String::new());
                        selected_wiki_page_id.set(String::new());
                        selected_wiki_content.set(String::new());
                        view_state.set(ViewState::WikiList);
                    },
                    "🗑 Delete"
                }

                button {
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        edit_wiki_content.set(String::new());
                        view_state.set(ViewState::WikiList);
                    },
                    "Cancel"
                }
            }
        }
    }
}
