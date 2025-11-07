// Wait for Tauri to be available
async function waitForTauri() {
    let attempts = 0;
    while (attempts < 50) {
        if (window.__TAURI_INTERNALS__) {
            return true;
        }
        await new Promise(resolve => setTimeout(resolve, 100));
        attempts++;
    }
    console.error('Tauri API not available');
    return false;
}

async function invoke(cmd, args = {}) {
    if (!window.__TAURI_INTERNALS__) {
        throw new Error('Tauri not available');
    }
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
}

async function listen(event, handler) {
    if (!window.__TAURI_INTERNALS__) {
        throw new Error('Tauri not available');
    }
    return window.__TAURI_INTERNALS__.event.listen(event, handler);
}

let currentView = 'wiki-list';
let currentPageId = '';
let currentUserId = '';
let currentPublicKey = '';
let navigationHistory = [];

function debug(msg) {
    console.log('[DEBUG]', msg);
}

// Initialize the app
async function init() {
    debug('Initializing app...');

    // Wait for Tauri to be ready
    const tauriReady = await waitForTauri();
    if (!tauriReady) {
        debug('ERROR: Tauri API not available');
        document.body.innerHTML = '<div style="padding: 20px; color: red;">Error: Tauri API not available</div>';
        return;
    }

    debug('Tauri API ready');

    // Listen for auth state changes
    try {
        debug('Setting up event listener...');
        await listen('auth-state-changed', async () => {
            console.log('Auth state changed event received');
            await updateAuthState();
        });
        debug('Event listener ready');
    } catch (error) {
        debug('ERROR setting up listener: ' + error);
    }

    // Set up event listeners
    debug('Setting up UI listeners...');
    setupEventListeners();

    // Update auth state
    debug('Calling updateAuthState...');
    await updateAuthState();

    // Poll auth state every 2 seconds to catch changes
    setInterval(async () => {
        await updateAuthState();
    }, 2000);

    // Initialize Lucide icons - wait for DOM and library to be ready
    const initIcons = () => {
        if (window.lucide && window.lucide.createIcons) {
            window.lucide.createIcons();
            console.log('Lucide icons initialized');
        } else {
            setTimeout(initIcons, 50);
        }
    };
    setTimeout(initIcons, 100);
}

function setupEventListeners() {
    // Create wiki view
    document.getElementById('create-new-btn').addEventListener('click', () => {
        showView('create-wiki');
    });

    document.getElementById('save-new-btn').addEventListener('click', async () => {
        const content = document.getElementById('create-content').value;
        try {
            await invoke('create_wiki', { content, filename: null });
            document.getElementById('create-content').value = '';
            showView('wiki-list');
            await loadWikiPages();
        } catch (error) {
            alert('Failed to create wiki: ' + error);
        }
    });

    document.getElementById('cancel-create-btn').addEventListener('click', () => {
        document.getElementById('create-content').value = '';
        showView('wiki-list');
    });

    // Edit wiki view
    document.getElementById('update-btn').addEventListener('click', async () => {
        const content = document.getElementById('edit-content').value;
        try {
            await invoke('update_wiki', { pageId: currentPageId, content });
            showView('wiki-list');
            await loadWikiPages();
        } catch (error) {
            alert('Failed to update wiki: ' + error);
        }
    });

    document.getElementById('cancel-edit-btn').addEventListener('click', () => {
        showView('wiki-list');
    });

    // View wiki
    document.getElementById('edit-wiki-btn').addEventListener('click', async () => {
        const content = await invoke('get_wiki_content', {
            userId: currentUserId,
            pageId: currentPageId
        });
        document.getElementById('edit-content').value = content;
        showView('edit-wiki');
    });

    document.getElementById('fork-wiki-btn').addEventListener('click', async () => {
        const content = await invoke('get_wiki_content', {
            userId: currentUserId,
            pageId: currentPageId
        });
        document.getElementById('create-content').value = content;
        showView('create-wiki');
    });

    document.getElementById('back-btn').addEventListener('click', () => {
        // If there's history, go back to the previous page
        if (navigationHistory.length > 0) {
            const previous = navigationHistory.pop();
            viewWiki(previous.userId, previous.pageId, false); // Don't add to history when going back
        } else {
            // No history, go to wiki list
            showView('wiki-list');
            navigationHistory = []; // Clear history when returning to list
        }
    });

    document.getElementById('share-link-btn').addEventListener('click', () => {
        const link = `[link](${currentUserId}/${currentPageId})`;
        navigator.clipboard.writeText(link);

        const btn = document.getElementById('share-link-btn');
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 2000);
    });

    // Delete button handler
    document.getElementById('delete-wiki-btn').addEventListener('click', async (e) => {
        console.log('Delete button clicked!', 'currentPageId:', currentPageId);
        e.preventDefault();
        e.stopPropagation();

        if (confirm('Are you sure you want to delete this wiki page?')) {
            try {
                console.log('Calling delete_wiki with pageId:', currentPageId);
                await invoke('delete_wiki', { pageId: currentPageId });
                console.log('Delete successful');
                showView('wiki-list');
                await loadWikiPages();
            } catch (error) {
                console.error('Delete error:', error);
                alert('Failed to delete wiki: ' + error);
            }
        }
    });

    // Collapsible headers
    document.querySelectorAll('.collapsible-header').forEach(header => {
        header.addEventListener('click', (e) => {
            const content = e.currentTarget.nextElementSibling;
            if (content) {
                content.classList.toggle('hidden');
            }
        });
    });
}

async function updateAuthState() {
    try {
        debug('Getting auth state...');
        const state = await invoke('get_auth_state');
        debug('Auth state: ' + state.type);

        // Hide all states
        document.getElementById('initializing-state').classList.add('hidden');
        document.getElementById('qr-state').classList.add('hidden');
        document.getElementById('error-state').classList.add('hidden');
        document.getElementById('auth-view').classList.add('hidden');
        document.getElementById('main-view').classList.add('hidden');

        if (state.type === 'Initializing') {
            debug('State: Initializing');
            document.getElementById('auth-view').classList.remove('hidden');
            document.getElementById('initializing-state').classList.remove('hidden');
        } else if (state.type === 'ShowingQR') {
            debug('State: ShowingQR - loading QR...');
            document.getElementById('auth-view').classList.remove('hidden');
            document.getElementById('qr-state').classList.remove('hidden');

            // Load QR code
            try {
                const qrImage = await invoke('get_qr_image');
                debug('QR loaded: ' + (qrImage ? qrImage.substring(0, 30) + '...' : 'EMPTY'));
                document.getElementById('qr-image').src = qrImage;
            } catch (error) {
                debug('ERROR loading QR: ' + error);
                document.getElementById('error-state').classList.remove('hidden');
                document.getElementById('error-message').textContent = 'Failed to load QR code: ' + error;
            }
        } else if (state.type === 'Authenticated') {
            debug('State: Authenticated');
            document.getElementById('main-view').classList.remove('hidden');
            currentPublicKey = state.public_key;
            await loadWikiPages();
        } else if (state.type === 'Error') {
            debug('State: Error - ' + state.message);
            document.getElementById('auth-view').classList.remove('hidden');
            document.getElementById('error-state').classList.remove('hidden');
            document.getElementById('error-message').textContent = state.message;
        }
    } catch (error) {
        debug('ERROR: Failed to get auth state: ' + error);
        document.getElementById('auth-view').classList.remove('hidden');
        document.getElementById('error-state').classList.remove('hidden');
        document.getElementById('error-message').textContent = 'Failed to get auth state: ' + error;
    }
}

async function loadWikiPages() {
    try {
        const pages = await invoke('get_wiki_pages');
        const wikiList = document.getElementById('wiki-list');

        if (pages.length === 0) {
            wikiList.innerHTML = '<p class="no-wikis">No wiki posts yet. Create your first one!</p>';
        } else {
            wikiList.innerHTML = pages.map(page => `
                <div class="wiki-item" onclick="viewWiki('${currentPublicKey}', '${page.id}')">
                    <span class="title">${page.title}</span>
                </div>
            `).join('');

            // Reinitialize Lucide icons after DOM update
            setTimeout(() => {
                if (window.lucide && window.lucide.createIcons) {
                    window.lucide.createIcons();
                }
            }, 50);
        }
    } catch (error) {
        console.error('Failed to load wiki pages:', error);
    }
}

async function viewWiki(userId, pageId, addToHistory = true) {
    // If navigating from wiki-list, clear history for a fresh start
    if (currentView === 'wiki-list') {
        navigationHistory = [];
    }
    // Add current page to history before navigating (if we're viewing a wiki currently)
    else if (addToHistory && currentView === 'view-wiki' && currentUserId && currentPageId) {
        // Only add if it's not the same as the last item in history
        const lastHistory = navigationHistory[navigationHistory.length - 1];
        if (!lastHistory || lastHistory.userId !== currentUserId || lastHistory.pageId !== currentPageId) {
            navigationHistory.push({ userId: currentUserId, pageId: currentPageId });
        }
    }

    currentUserId = userId;
    currentPageId = pageId;

    try {
        // Load content
        const content = await invoke('get_wiki_content', { userId, pageId });

        // Render markdown
        const renderedHtml = marked.parse(content);
        document.getElementById('wiki-content').innerHTML = renderedHtml;

        // Handle custom Pubky Wiki links
        document.getElementById('wiki-content').querySelectorAll('a').forEach(link => {
            const href = link.getAttribute('href');
            if (href && !href.startsWith('http') && !href.startsWith('#')) {
                // Check if it matches userId/pageId pattern
                const match = href.match(/^([^\/]+)\/(.+)$/);
                if (match) {
                    const [, linkUserId, linkPageId] = match;
                    link.onclick = (e) => {
                        e.preventDefault();
                        viewWiki(linkUserId, linkPageId);
                    };
                }
            }
        });

        // Update page details
        document.getElementById('page-details-id').textContent = `Page ID: ${pageId}`;
        document.getElementById('page-details-user').textContent = `User ID: ${userId}`;

        // Load forks
        const forks = await invoke('discover_forks', { pageId });
        const forksHeader = document.getElementById('forks-header');
        forksHeader.innerHTML = `<i data-lucide="git-branch"></i><span>Available Forks (${forks.length})</span>`;
        if (window.lucide && window.lucide.createIcons) {
            window.lucide.createIcons();
        }

        const forksList = document.getElementById('forks-list');
        forksList.innerHTML = forks.map(fork => {
            const [forkUserId, forkPageId] = fork.split('/');
            const isCurrent = forkUserId === userId;
            const isOwn = forkUserId === currentPublicKey;

            let displayName;
            if (isOwn) {
                displayName = 'Your fork' + (isCurrent ? ' (viewing)' : '');
            } else {
                // Truncate the public key: show first 8 and last 4 characters
                const truncated = forkUserId.substring(0, 8) + '...' + forkUserId.substring(forkUserId.length - 4);
                displayName = truncated + (isCurrent ? ' (viewing)' : '');
            }

            return `
                <div class="my-2">
                    <button onclick="viewWiki('${forkUserId}', '${forkPageId}')" class="w-full flex items-center px-4 py-3 bg-lime-400 hover:bg-lime-500 text-gray-900 font-semibold rounded-lg transition-all hover:translate-x-0.5">${displayName}</button>
                </div>
            `;
        }).join('');

        // Show/hide edit, delete, and fork buttons
        const isOwnPage = userId === currentPublicKey;
        if (isOwnPage) {
            document.getElementById('edit-wiki-btn').classList.remove('hidden');
            document.getElementById('delete-wiki-btn').classList.remove('hidden');
            document.getElementById('fork-wiki-btn').classList.add('hidden');
        } else {
            document.getElementById('edit-wiki-btn').classList.add('hidden');
            document.getElementById('delete-wiki-btn').classList.add('hidden');
            document.getElementById('fork-wiki-btn').classList.remove('hidden');
        }

        showView('view-wiki');

        // Reinitialize Lucide icons after DOM update
        if (window.lucide && window.lucide.createIcons) {
            window.lucide.createIcons();
        }
    } catch (error) {
        alert('Failed to load wiki content: ' + error);
    }
}

function showView(viewName) {
    // Hide all content views
    document.querySelectorAll('.content-view').forEach(view => {
        view.classList.add('hidden');
    });

    // Show the selected view
    document.getElementById(`${viewName}-view`).classList.remove('hidden');
    currentView = viewName;
}

// Make viewWiki available globally
window.viewWiki = viewWiki;

// Initialize the app when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
