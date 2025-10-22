use egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use super::{convert_custom_links, PageStore};

/// WikiView manages the wiki page rendering and navigation
pub struct WikiView {
    current_page: String,
    page_store: PageStore,
    cache: CommonMarkCache,
}

impl WikiView {
    /// Creates a new WikiView starting at the home page
    pub fn new() -> Self {
        log::info!("Creating new WikiView");
        Self {
            current_page: "home".to_string(),
            page_store: PageStore::with_test_pages(),
            cache: CommonMarkCache::default(),
        }
    }

    /// Renders the wiki view in the given UI context
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Show current page breadcrumb
            ui.horizontal(|ui| {
                ui.label("📄 Current page:");
                ui.label(
                    egui::RichText::new(&self.current_page)
                        .monospace()
                        .color(egui::Color32::from_rgb(100, 149, 237)),
                );
            });
            ui.add_space(10.0);

            // Get the current page content
            if let Some(content) = self.page_store.get_page(&self.current_page) {
                // Convert custom link format to standard markdown
                let converted_content = convert_custom_links(content);

                // Render the markdown content with custom link handler
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let viewer = CommonMarkViewer::new();
                        viewer.show(ui, &mut self.cache, &converted_content);
                    });
            } else {
                ui.colored_label(egui::Color32::RED, "Page not found!");
                ui.add_space(10.0);
                if ui.button("Go to Home").clicked() {
                    self.navigate_to("home");
                }
            }
        });
        
        // Intercept link clicks by checking the output commands
        let clicked_urls: Vec<String> = ui.ctx().output_mut(|o| {
            let mut urls = Vec::new();
            // Drain commands to prevent external opening and capture URLs
            o.commands.retain(|cmd| {
                if let egui::output::OutputCommand::OpenUrl(open_url) = cmd {
                    log::info!("🔗 Intercepted link click: {}", open_url.url);
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
            log::info!("📄 Navigating to: {}", url);
            self.navigate_to(&url);
        }
    }

    /// Navigates to a different page
    fn navigate_to(&mut self, page_id: &str) {
        log::info!("Navigate to page: {} -> {}", self.current_page, page_id);
        self.current_page = page_id.to_string();
        // Clear the cache to ensure fresh rendering
        self.cache = CommonMarkCache::default();
    }

    /// Gets the current page ID
    pub fn current_page(&self) -> &str {
        &self.current_page
    }
}

impl Default for WikiView {
    fn default() -> Self {
        Self::new()
    }
}

