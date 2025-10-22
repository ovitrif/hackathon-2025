use crate::{PubkyApp, ViewState};

use eframe::egui::{Context, Ui};
use egui_commonmark::CommonMarkViewer;
use pubky::PublicStorage;

#[derive(Debug, Clone)]
pub enum DiffLine {
    Unchanged(String),
    Added(String),
    Removed(String),
    Modified { old: String, new: String },
}

/// Compute the Longest Common Subsequence (LCS) for diff algorithm
fn lcs_length(text1: &[&str], text2: &[&str]) -> Vec<Vec<usize>> {
    let m = text1.len();
    let n = text2.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if text1[i - 1] == text2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    dp
}

/// Calculate similarity between two strings (0.0 = completely different, 1.0 = identical)
fn line_similarity(s1: &str, s2: &str) -> f32 {
    let s1_words: Vec<&str> = s1.split_whitespace().collect();
    let s2_words: Vec<&str> = s2.split_whitespace().collect();
    
    if s1_words.is_empty() && s2_words.is_empty() {
        return 1.0;
    }
    if s1_words.is_empty() || s2_words.is_empty() {
        return 0.0;
    }
    
    let common_words = s1_words.iter().filter(|w| s2_words.contains(w)).count();
    let max_words = s1_words.len().max(s2_words.len());
    
    common_words as f32 / max_words as f32
}

/// Compute diff between two texts using LCS algorithm, then detect modifications
pub fn compute_diff(text1: &str, text2: &str) -> Vec<DiffLine> {
    let lines1: Vec<&str> = text1.lines().collect();
    let lines2: Vec<&str> = text2.lines().collect();

    let dp = lcs_length(&lines1, &lines2);
    let mut result = Vec::new();

    let mut i = lines1.len();
    let mut j = lines2.len();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 && lines1[i - 1] == lines2[j - 1] {
            result.push(DiffLine::Unchanged(lines1[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            result.push(DiffLine::Added(lines2[j - 1].to_string()));
            j -= 1;
        } else if i > 0 {
            result.push(DiffLine::Removed(lines1[i - 1].to_string()));
            i -= 1;
        }
    }

    result.reverse();
    
    // Post-process to detect modifications (removed followed by added with high similarity)
    let mut processed = Vec::new();
    let mut i = 0;
    
    while i < result.len() {
        match &result[i] {
            DiffLine::Removed(old) => {
                // Check if next line is Added and similar enough
                if i + 1 < result.len() {
                    if let DiffLine::Added(new) = &result[i + 1] {
                        let similarity = line_similarity(old, new);
                        // If similarity > 30%, treat as modification
                        if similarity > 0.3 {
                            processed.push(DiffLine::Modified {
                                old: old.clone(),
                                new: new.clone(),
                            });
                            i += 2; // Skip both removed and added
                            continue;
                        }
                    }
                }
                processed.push(result[i].clone());
                i += 1;
            }
            _ => {
                processed.push(result[i].clone());
                i += 1;
            }
        }
    }
    
    processed
}

pub(crate) fn update(
    app: &mut PubkyApp,
    pub_storage: &PublicStorage,
    _ctx: &Context,
    ui: &mut Ui,
) {
    ui.heading("Compare Articles");
    ui.add_space(20.0);

    // Show which article is which with clear labels
    ui.horizontal(|ui| {
        ui.label("Article 1:");
        ui.monospace(&app.comparison_title_1);
    });
    ui.horizontal(|ui| {
        ui.label("Article 2:");
        ui.monospace(&app.comparison_title_2);
    });
    ui.add_space(20.0);

    // Load content for both articles if not already loaded
    if app.comparison_content_1.is_empty() && app.selected_for_compare.len() >= 1 {
        if let Some((user_id, page_id)) = app.selected_for_compare.get(0) {
            let public_storage_clone = pub_storage.clone();
            let path = format!("pubky{}/pub/wiki.app/{}", user_id, page_id);

            let get_path_fut = public_storage_clone.get(&path);
            match app.rt.block_on(get_path_fut) {
                Ok(response) => {
                    let response_text_fut = response.text();
                    match app.rt.block_on(response_text_fut) {
                        Ok(text) => {
                            app.comparison_content_1 = text;
                        }
                        Err(e) => {
                            app.comparison_content_1 = format!("Error loading article 1: {}", e);
                        }
                    }
                }
                Err(e) => {
                    app.comparison_content_1 = format!("Error fetching article 1 at {}: {}", path, e);
                }
            }
        }
    }

    if app.comparison_content_2.is_empty() && app.selected_for_compare.len() >= 2 {
        if let Some((user_id, page_id)) = app.selected_for_compare.get(1) {
            let public_storage_clone = pub_storage.clone();
            let path = format!("pubky{}/pub/wiki.app/{}", user_id, page_id);

            let get_path_fut = public_storage_clone.get(&path);
            match app.rt.block_on(get_path_fut) {
                Ok(response) => {
                    let response_text_fut = response.text();
                    match app.rt.block_on(response_text_fut) {
                        Ok(text) => {
                            app.comparison_content_2 = text;
                        }
                        Err(e) => {
                            app.comparison_content_2 = format!("Error loading article 2: {}", e);
                        }
                    }
                }
                Err(e) => {
                    app.comparison_content_2 = format!("Error fetching article 2 at {}: {}", path, e);
                }
            }
        }
    }

    // Compute diff
    let diff_lines = compute_diff(&app.comparison_content_1, &app.comparison_content_2);

    // Render diff with scrollable area
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for diff_line in diff_lines.iter() {
                match diff_line {
                    DiffLine::Unchanged(line) => {
                        // Skip empty/whitespace-only lines
                        if line.trim().is_empty() {
                            continue;
                        }
                        // Render unchanged line normally
                        egui::Frame::new().show(ui, |ui| {
                            CommonMarkViewer::new()
                                .max_image_width(Some(512))
                                .show(ui, &mut app.cache, line);
                        });
                    }
                    DiffLine::Removed(line) => {
                        // Skip empty/whitespace-only lines
                        if line.trim().is_empty() {
                            continue;
                        }
                        // Render removed line with 7.5% opacity red background (alpha 0.075)
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(255, 0, 0, 19))
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                CommonMarkViewer::new()
                                    .max_image_width(Some(512))
                                    .show(ui, &mut app.cache, line);
                            });
                    }
                    DiffLine::Added(line) => {
                        // Skip empty/whitespace-only lines
                        if line.trim().is_empty() {
                            continue;
                        }
                        // Render added line with 7.5% opacity green background (alpha 0.075)
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 255, 0, 19))
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                CommonMarkViewer::new()
                                    .max_image_width(Some(512))
                                    .show(ui, &mut app.cache, line);
                            });
                    }
                    DiffLine::Modified { old, new } => {
                        // Skip if both are empty/whitespace-only
                        if old.trim().is_empty() && new.trim().is_empty() {
                            continue;
                        }
                        // Render modified line showing both old (red) and new (green)
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(255, 0, 0, 19))
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                CommonMarkViewer::new()
                                    .max_image_width(Some(512))
                                    .show(ui, &mut app.cache, old);
                            });
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 255, 0, 19))
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                CommonMarkViewer::new()
                                    .max_image_width(Some(512))
                                    .show(ui, &mut app.cache, new);
                            });
                    }
                }
            }
        });

    ui.add_space(20.0);

    // Go back button
    if ui.button("Go back").clicked() {
        // Clear comparison state
        app.comparison_mode = false;
        app.selected_for_compare.clear();
        app.comparison_content_1.clear();
        app.comparison_content_2.clear();
        app.comparison_title_1.clear();
        app.comparison_title_2.clear();
        app.view_state = ViewState::WikiList;
    }
}
