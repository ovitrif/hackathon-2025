use crate::{utils::extract_details_wiki_url, ViewState};
use dioxus::prelude::*;
use pubky::{PubkySession, PublicStorage};

pub fn ViewWiki(
    session: PubkySession,
    pub_storage: PublicStorage,
    mut view_state: Signal<ViewState>,
    mut selected_wiki_page_id: Signal<String>,
    mut selected_wiki_content: Signal<String>,
    mut selected_wiki_user_id: Signal<String>,
    mut selected_wiki_fork_urls: Signal<Vec<String>>,
    mut edit_wiki_content: Signal<String>,
    mut forked_from_page_id: Signal<Option<String>>,
    mut show_copy_tooltip: Signal<bool>,
) -> Element {
    let page_id = selected_wiki_page_id();
    let user_id = selected_wiki_user_id();
    let content = selected_wiki_content();
    
    // Fetch content if empty
    {
        let page_id_clone = page_id.clone();
        let user_id_clone = user_id.clone();
        let content_clone = content.clone();
        let pub_storage_clone = pub_storage.clone();
        use_effect(move || {
            if content_clone.is_empty() && !page_id_clone.is_empty() && !user_id_clone.is_empty() {
                let path = format!("pubky://{user_id_clone}/pub/wiki.app/{page_id_clone}");
                let pub_storage_c = pub_storage_clone.clone();
                
                spawn(async move {
                    match pub_storage_c.get(&path).await {
                        Ok(response) => {
                            match response.text().await {
                                Ok(text) => {
                                    selected_wiki_content.set(text);
                                }
                                Err(e) => {
                                    selected_wiki_content.set(format!("Error reading content: {e}"));
                                }
                            }
                        }
                        Err(e) => {
                            selected_wiki_content.set(format!("Error fetching path {path}: {e}"));
                        }
                    }
                });
            }
        });
    }

    let pk = session.info().public_key().to_string();
    let is_own_page = user_id == pk;
    let fork_urls = selected_wiki_fork_urls();

    // Convert markdown to HTML
    let html_content = markdown::to_html(&content);

    let session_for_forks = session.clone();
    let pub_storage_for_forks = pub_storage.clone();

    rsx! {
        div { class: "view-wiki",
            h2 { "View Wiki Post" }

            details { class: "details-section",
                summary { "📋 Page Details" }
                p { "Page ID: ", code { "{page_id}" } }
                p { "User ID: ", code { "{user_id}" } }
            }

            details { class: "details-section",
                summary { "🔀 Available Forks ({fork_urls.len()})" }
                for fork_link in fork_urls.clone() {
                    {
                        if let Some((fork_user_pk, fork_page_id)) = extract_details_wiki_url(&fork_link) {
                            let is_current = fork_user_pk == user_id;
                            let label = if is_current {
                                format!("Fork: {fork_user_pk} (current)")
                            } else {
                                format!("Fork: {fork_user_pk}")
                            };
                            
                            let session_clone = session_for_forks.clone();
                            let pub_storage_clone = pub_storage_for_forks.clone();
                            
                            rsx! {
                                button {
                                    class: "btn btn-secondary",
                                    onclick: move |_| {
                                        selected_wiki_user_id.set(fork_user_pk.clone());
                                        selected_wiki_page_id.set(fork_page_id.clone());
                                        selected_wiki_content.set(String::new());
                                        
                                        let session_c = session_clone.clone();
                                        let page_id_c = fork_page_id.clone();
                                        let pub_storage_c = pub_storage_clone.clone();
                                        
                                        spawn(async move {
                                            let fork_urls = crate::discover_fork_urls(&session_c, &pub_storage_c, &page_id_c).await;
                                            selected_wiki_fork_urls.set(fork_urls);
                                        });
                                        
                                        view_state.set(ViewState::ViewWiki);
                                    },
                                    "{label}"
                                }
                            }
                        } else {
                            None
                        }
                    }
                }
            }

            button {
                class: "btn btn-secondary",
                onclick: move |_| {
                    // Copy to clipboard (need to use JS interop for web)
                    let link_text = format!("[link]({user_id}/{page_id})");
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        // For desktop, we could use clipboard crate
                        log::info!("Share link: {}", link_text);
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        // For web, use JS clipboard API
                        log::info!("Share link: {}", link_text);
                    }
                    show_copy_tooltip.set(true);
                },
                "🔗 Share Page Link"
            }

            if show_copy_tooltip() {
                span { class: "tooltip", "Copied!" }
            }

            div { class: "content-section",
                h3 { "Content" }
                div {
                    class: "markdown-content",
                    dangerous_inner_html: "{html_content}"
                }
            }

            div { class: "button-group",
                if is_own_page {
                    {
                        let content_clone = content.clone();
                        rsx! {
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    edit_wiki_content.set(content_clone.clone());
                                    view_state.set(ViewState::EditWiki);
                                },
                                "✏ Edit"
                            }
                        }
                    }
                }

                if !is_own_page {
                    {
                        let content_clone = content.clone();
                        let page_id_clone = page_id.clone();
                        rsx! {
                            button {
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    edit_wiki_content.set(content_clone.clone());
                                    forked_from_page_id.set(Some(page_id_clone.clone()));
                                    view_state.set(ViewState::CreateWiki);
                                },
                                "🍴 Fork"
                            }
                        }
                    }
                }

                button {
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        selected_wiki_page_id.set(String::new());
                        selected_wiki_content.set(String::new());
                        selected_wiki_fork_urls.set(Vec::new());
                        view_state.set(ViewState::WikiList);
                    },
                    "← Back"
                }
            }
        }
    }
}
