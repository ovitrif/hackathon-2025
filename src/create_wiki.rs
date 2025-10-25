use crate::{create_wiki_post, utils::extract_title, AuthState, ViewState};
use dioxus::prelude::*;
use pubky::PubkySession;

pub fn CreateWiki(
    session: PubkySession,
    mut view_state: Signal<ViewState>,
    mut edit_wiki_content: Signal<String>,
    mut forked_from_page_id: Signal<Option<String>>,
    auth_state: Signal<AuthState>,
) -> Element {
    rsx! {
        div { class: "create-wiki",
            h2 { "Create New Wiki Page" }

            label { "Content:" }
            textarea {
                class: "wiki-editor",
                value: "{edit_wiki_content}",
                rows: "15",
                oninput: move |evt| edit_wiki_content.set(evt.value()),
                placeholder: "# Your Wiki Page Title\n\nYour content here..."
            }

            div { class: "button-group",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let session_clone = session.clone();
                        let content = edit_wiki_content();
                        let filename = forked_from_page_id();
                        let mut auth_state_clone = auth_state;
                        
                        spawn(async move {
                            match create_wiki_post(&session_clone, &content, filename.as_deref()).await {
                                Ok(wiki_page_path) => {
                                    log::info!("Created wiki post at: {}", wiki_page_path);

                                    // Update file cache
                                    if let AuthState::Authenticated { session, ref mut file_cache, .. } = &mut *auth_state_clone.write() {
                                        let own_user_pk = session.info().public_key().to_string();
                                        let file_url = format!("pubky://{own_user_pk}{wiki_page_path}");
                                        let file_title = extract_title(&content);
                                        file_cache.insert(file_url, file_title.to_string());
                                    }
                                }
                                Err(e) => log::error!("Failed to create wiki post: {e}"),
                            }
                        });

                        edit_wiki_content.set(String::new());
                        forked_from_page_id.set(None);
                        view_state.set(ViewState::WikiList);
                    },
                    "💾 Save"
                }

                button {
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        edit_wiki_content.set(String::new());
                        forked_from_page_id.set(None);
                        view_state.set(ViewState::WikiList);
                    },
                    "Cancel"
                }
            }
        }
    }
}
