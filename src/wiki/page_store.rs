use std::collections::HashMap;

/// Stores wiki pages in memory with their markdown content
#[derive(Debug, Clone)]
pub struct PageStore {
    pages: HashMap<String, String>,
}

impl PageStore {
    /// Creates a new empty page store
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
        }
    }

    /// Initializes the store with test pages
    pub fn with_test_pages() -> Self {
        let mut store = Self::new();
        store.initialize_test_pages();
        store
    }

    /// Gets a page by its ID
    pub fn get_page(&self, page_id: &str) -> Option<&String> {
        self.pages.get(page_id)
    }

    /// Adds or updates a page
    pub fn set_page(&mut self, page_id: String, content: String) {
        self.pages.insert(page_id, content);
    }

    /// Initializes with test content
    fn initialize_test_pages(&mut self) {
        // Home page with custom link format
        let home_content = r#"# Welcome to the Wiki

This is your personal wiki space!

Check out (Alice's Page)[alice_user_id/550e8400-e29b-41d4-a716-446655440000] to see another page.

You can also visit (Bob's Notes)[bob_user_id/660e9500-f39c-52e5-b827-557766551111].
"#;

        // Alice's page with custom link format
        let alice_content = r#"# Alice's Page

Welcome to Alice's personal page! This is where Alice keeps her notes and ideas.

## Quick Links

- (Go back Home)[home]
- (Check out Bob's Notes)[bob_user_id/660e9500-f39c-52e5-b827-557766551111]

## About Alice

Alice is exploring the decentralized wiki world!
"#;

        // Bob's page
        let bob_content = r#"# Bob's Notes

Welcome to Bob's note collection.

## Navigation

- (Back to Home)[home]
- (Visit Alice's Page)[alice_user_id/550e8400-e29b-41d4-a716-446655440000]

## Bob's Thoughts

Just testing out this wiki system. Pretty cool!
"#;

        self.set_page("home".to_string(), home_content.to_string());
        self.set_page(
            "alice_user_id/550e8400-e29b-41d4-a716-446655440000".to_string(),
            alice_content.to_string(),
        );
        self.set_page(
            "bob_user_id/660e9500-f39c-52e5-b827-557766551111".to_string(),
            bob_content.to_string(),
        );
    }
}

impl Default for PageStore {
    fn default() -> Self {
        Self::new()
    }
}

