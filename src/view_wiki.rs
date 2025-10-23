use crate::{utils::extract_details_wiki_url, PubkyApp, ViewState};

use eframe::egui::{Context, Ui};
use egui::CollapsingHeader;
use egui_commonmark::CommonMarkViewer;
use pubky::{PubkySession, PublicStorage};

pub(crate) fn update(
    app: &mut PubkyApp,
    session: &PubkySession,
    pub_storage: &PublicStorage,
    ctx: &Context,
    ui: &mut Ui,
) {
    ui.label(egui::RichText::new("View Wiki Post").size(20.0).strong());
    ui.add_space(25.0);

    CollapsingHeader::new(egui::RichText::new("📋 Page Details").size(15.0)).show(ui, |ui| {
        ui.add_space(5.0);
        ui.label(egui::RichText::new(format!("Page ID: {}", &app.selected_wiki_page_id)).monospace());
        ui.label(egui::RichText::new(format!("User ID: {}", &app.selected_wiki_user_id)).monospace());
    });

    ui.add_space(10.0);
    let fork_links = app.selected_wiki_fork_urls.clone();
    CollapsingHeader::new(egui::RichText::new(format!("🔀 Available Forks ({})", fork_links.len())).size(15.0)).show(ui, |ui| {
        ui.add_space(5.0);
        for fork_link in fork_links {
            if let Some((user_pk, page_id)) = extract_details_wiki_url(&fork_link) {
                let is_current_article = &app.selected_wiki_user_id == &user_pk 
                    && &app.selected_wiki_page_id == &page_id;

                ui.horizontal(|ui| {
                    // Truncate user_pk for display
                    let display_pk = if user_pk.len() > 20 {
                        format!("{}...", &user_pk[..20])
                    } else {
                        user_pk.clone()
                    };
                    
                    let btn_text = if is_current_article {
                        format!("📄 {} (current)", display_pk)
                    } else {
                        format!("📄 {}", display_pk)
                    };
                    
                    let view_button = ui.add_sized(
                        [300.0, 30.0],
                        egui::Button::new(egui::RichText::new(btn_text).size(14.0))
                    );
                    
                    if view_button.clicked() {
                        app.navigate_to_view_wiki_page(&user_pk, &page_id, session, pub_storage);
                    }
                    
                    // Compare button - don't show for the exact same article we're viewing
                    if !is_current_article {
                        ui.add_space(5.0);
                        let compare_button = ui.add_sized(
                            [100.0, 30.0],
                            egui::Button::new(egui::RichText::new("🔍 Compare").size(14.0))
                        );
                        
                        if compare_button.clicked() {
                            // Set up comparison: current article vs this fork
                            app.selected_for_compare = vec![
                                (app.selected_wiki_user_id.clone(), app.selected_wiki_page_id.clone()),
                                (user_pk.clone(), page_id.clone()),
                            ];
                            app.comparison_title_1 = app.selected_wiki_page_id.clone();
                            app.comparison_title_2 = page_id.clone();
                            app.comparison_content_1 = app.selected_wiki_content.clone();
                            app.comparison_content_2 = String::new(); // Will be loaded in compare view
                            app.view_state = ViewState::CompareWiki;
                        }
                    }
                });
                ui.add_space(5.0);
            }
        }
    });

    ui.add_space(15.0);
    // Add "Share Page Link" button with tooltip support
    let share_button = ui.add_sized(
        [180.0, 35.0],
        egui::Button::new(egui::RichText::new("🔗 Share Page Link").size(15.0))
    );

    // Show tooltip when hovering after copy
    if app.show_copy_tooltip {
        share_button.show_tooltip_text("Copied");
    }

    if share_button.clicked() {
        let user_id = &app.selected_wiki_user_id;
        let page_id = &app.selected_wiki_page_id;
        ctx.copy_text(format!("[link]({user_id}/{page_id})"));
        app.show_copy_tooltip = true;
    }

    // Reset tooltip if button is not being hovered
    if !share_button.hovered() && app.show_copy_tooltip {
        app.show_copy_tooltip = false;
    }

    ui.add_space(15.0);

    // Display content in a scrollable area
    ui.separator();
    ui.add_space(15.0);
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            // Try to fetch content if empty
            if app.selected_wiki_content.is_empty()
                && !app.selected_wiki_page_id.is_empty()
                && !app.selected_wiki_user_id.is_empty()
            {
                let public_storage_clone = pub_storage.clone();
                let path_clone = app.selected_wiki_page_id.clone();
                let user_id = app.selected_wiki_user_id.clone();

                let path = format!("pubky{user_id}/pub/wiki.app/{path_clone}");

                // Synchronously fetch the content
                let get_path_fut = public_storage_clone.get(&path);
                let fetched_content = match app.rt.block_on(get_path_fut) {
                    Ok(response) => match app.rt.block_on(response.text()) {
                        Ok(text) => text,
                        Err(e) => format!("Error reading content: {e}"),
                    },
                    Err(e) => format!("Error fetching path {path}: {e}"),
                };
                app.selected_wiki_content = fetched_content;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                CommonMarkViewer::new().max_image_width(Some(512)).show(
                    ui,
                    &mut app.cache,
                    &app.selected_wiki_content.as_str(),
                );
            });

            // Intercept link clicks by checking the output commands
            let clicked_urls: Vec<String> = ui.ctx().output_mut(|o| {
                let mut urls = Vec::new();
                // Drain commands to prevent external opening and capture URLs
                o.commands.retain(|cmd| {
                    if let egui::output::OutputCommand::OpenUrl(open_url) = cmd {
                        log::info!("Intercepted link click: {}", open_url.url);
                        urls.push(open_url.url.to_string());
                        false // Remove this command to prevent external opening
                    } else {
                        true // Keep other commands
                    }
                });
                urls
            });

            // Navigate to clicked URLs
            for url in clicked_urls {
                if let Some((user_pk, page_id)) = extract_details_wiki_url(&url) {
                    app.navigate_to_view_wiki_page(&user_pk, &page_id, session, pub_storage);
                }
            }
        });

    ui.add_space(25.0);

    // Check if this is the user's own page
    let pk = session.info().public_key();
    let is_own_page = app.selected_wiki_user_id == pk.to_string();

    ui.horizontal(|ui| {
        // Show Edit button only for own pages
        if is_own_page {
            let edit_button = ui.add_sized(
                [120.0, 35.0],
                egui::Button::new(egui::RichText::new("✏ Edit").size(15.0))
            );
            if edit_button.clicked() {
                app.navigate_to_edit_selected_wiki_page();
            }
            ui.add_space(10.0);
        }

        // Fork button - available for only when viewing other user's pages
        if !is_own_page {
            let fork_button = ui.add_sized(
                [120.0, 35.0],
                egui::Button::new(egui::RichText::new("🍴 Fork").size(15.0))
            );
            if fork_button.clicked() {
                app.edit_wiki_content = app.selected_wiki_content.clone();
                app.forked_from_page_id = Some(app.selected_wiki_page_id.clone());
                app.view_state = ViewState::CreateWiki;
            }
            ui.add_space(10.0);
        }

        // Go back button
        let back_button = ui.add_sized(
            [120.0, 35.0],
            egui::Button::new(egui::RichText::new("← Back").size(15.0))
        );
        if back_button.clicked() {
            app.selected_wiki_page_id.clear();
            app.selected_wiki_content.clear();
            app.selected_wiki_fork_urls.clear();
            app.view_state = ViewState::WikiList;
        }
    });
}
