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

    document.getElementById('delete-btn').addEventListener('click', async () => {
        if (confirm('Are you sure you want to delete this wiki page?')) {
            try {
                await invoke('delete_wiki', { pageId: currentPageId });
                showView('wiki-list');
                await loadWikiPages();
            } catch (error) {
                alert('Failed to delete wiki: ' + error);
            }
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
        showView('wiki-list');
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

    // Collapsible headers
    document.querySelectorAll('.collapsible-header').forEach(header => {
        header.addEventListener('click', (e) => {
            const content = e.target.nextElementSibling;
            if (content.style.display === 'none') {
                content.style.display = 'block';
            } else {
                content.style.display = 'none';
            }
        });
    });
}

async function updateAuthState() {
    try {
        debug('Getting auth state...');
        const state = await invoke('get_auth_state');
        debug('Auth state: ' + state.type);

        // Hide all auth states
        document.getElementById('initializing-state').style.display = 'none';
        document.getElementById('qr-state').style.display = 'none';
        document.getElementById('error-state').style.display = 'none';
        document.getElementById('auth-view').style.display = 'none';
        document.getElementById('main-view').style.display = 'none';

        if (state.type === 'Initializing') {
            debug('State: Initializing');
            document.getElementById('auth-view').style.display = 'block';
            document.getElementById('initializing-state').style.display = 'block';
        } else if (state.type === 'ShowingQR') {
            debug('State: ShowingQR - loading QR...');
            document.getElementById('auth-view').style.display = 'block';
            document.getElementById('qr-state').style.display = 'block';

            // Load QR code
            try {
                const qrImage = await invoke('get_qr_image');
                debug('QR loaded: ' + (qrImage ? qrImage.substring(0, 30) + '...' : 'EMPTY'));
                document.getElementById('qr-image').src = qrImage;
            } catch (error) {
                debug('ERROR loading QR: ' + error);
                document.getElementById('error-state').style.display = 'block';
                document.getElementById('error-message').textContent = 'Failed to load QR code: ' + error;
            }
        } else if (state.type === 'Authenticated') {
            debug('State: Authenticated');
            document.getElementById('main-view').style.display = 'block';
            currentPublicKey = state.public_key;
            await loadWikiPages();
        } else if (state.type === 'Error') {
            debug('State: Error - ' + state.message);
            document.getElementById('auth-view').style.display = 'block';
            document.getElementById('error-state').style.display = 'block';
            document.getElementById('error-message').textContent = state.message;
        }
    } catch (error) {
        debug('ERROR: Failed to get auth state: ' + error);
        document.getElementById('auth-view').style.display = 'block';
        document.getElementById('error-state').style.display = 'block';
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

async function viewWiki(userId, pageId) {
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
                <div class="fork-item">
                    <button onclick="viewWiki('${forkUserId}', '${forkPageId}')">${displayName}</button>
                </div>
            `;
        }).join('');

        // Show/hide edit and fork buttons
        const isOwnPage = userId === currentPublicKey;
        document.getElementById('edit-wiki-btn').style.display = isOwnPage ? 'inline-block' : 'none';
        document.getElementById('fork-wiki-btn').style.display = !isOwnPage ? 'inline-block' : 'none';

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
        view.style.display = 'none';
    });

    // Show the selected view
    document.getElementById(`${viewName}-view`).style.display = 'block';
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
