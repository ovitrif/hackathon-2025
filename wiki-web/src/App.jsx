import { useState, useEffect } from 'react';
import AuthView from './components/AuthView';
import WikiList from './components/WikiList';
import CreateWiki from './components/CreateWiki';
import EditWiki from './components/EditWiki';
import ViewWiki from './components/ViewWiki';

const APP_NAME = "Pubky Wiki";

// Auth states
const AUTH_STATE = {
  INITIALIZING: 'initializing',
  SHOWING_QR: 'showing_qr',
  AUTHENTICATED: 'authenticated',
  ERROR: 'error'
};

// View states
const VIEW_STATE = {
  WIKI_LIST: 'wiki_list',
  CREATE_WIKI: 'create_wiki',
  VIEW_WIKI: 'view_wiki',
  EDIT_WIKI: 'edit_wiki'
};

// Mock session for demonstration
const createMockSession = () => ({
  info: () => ({
    publicKey: () => 'mock-user-public-key-12345678'
  }),
  storage: () => ({
    put: async (path, content) => {
      console.log('Mock PUT:', path, content);
      // Save to localStorage for demo
      const key = `wiki-${path}`;
      localStorage.setItem(key, content);
      return Promise.resolve();
    },
    delete: async (path) => {
      console.log('Mock DELETE:', path);
      const key = `wiki-${path}`;
      localStorage.removeItem(key);
      return Promise.resolve();
    },
    list: (path) => ({
      send: async () => {
        console.log('Mock LIST:', path);
        // Get all wiki entries from localStorage
        const entries = [];
        for (let i = 0; i < localStorage.length; i++) {
          const key = localStorage.key(i);
          if (key.startsWith('wiki-/pub/wiki.app/')) {
            const wikiPath = key.replace('wiki-', '');
            entries.push({
              toPubkyUrl: () => `pubky://mock-user-public-key-12345678${wikiPath}`
            });
          }
        }
        return entries;
      }
    })
  })
});

// Mock public storage for demonstration
const createMockPublicStorage = () => ({
  get: async (url) => {
    console.log('Mock GET:', url);
    // Extract path from URL
    const path = url.replace(/^pubky:\/\/[^\/]+/, '');
    const key = `wiki-${path}`;
    const content = localStorage.getItem(key) || '# Not Found\n\nThis page does not exist.';
    return {
      text: async () => content
    };
  }
});

function App() {
  // Auth state
  const [authState, setAuthState] = useState(AUTH_STATE.INITIALIZING);
  const [authUrl] = useState('pubkyauth://demo-auth-url-for-testing');
  const [error, setError] = useState('');
  const [session, setSession] = useState(null);
  const [publicStorage, setPublicStorage] = useState(null);
  const [fileCache, setFileCache] = useState({});

  // View state
  const [viewState, setViewState] = useState(VIEW_STATE.WIKI_LIST);
  const [editWikiContent, setEditWikiContent] = useState('');
  const [selectedWikiPageId, setSelectedWikiPageId] = useState('');
  const [selectedWikiContent, setSelectedWikiContent] = useState('');
  const [selectedWikiUserId, setSelectedWikiUserId] = useState('');
  const [selectedWikiForkUrls, setSelectedWikiForkUrls] = useState([]);
  const [forkedFromPageId, setForkedFromPageId] = useState(null);
  const [showCopyTooltip, setShowCopyTooltip] = useState(false);

  // Initialize authentication (mock for demo)
  useEffect(() => {
    const initAuth = async () => {
      try {
        // Show QR code for a brief moment
        setAuthState(AUTH_STATE.SHOWING_QR);

        // Simulate authentication after 2 seconds for demo purposes
        setTimeout(async () => {
          const mockSession = createMockSession();
          const mockPubStorage = createMockPublicStorage();

          setSession(mockSession);
          setPublicStorage(mockPubStorage);

          // Fetch files
          await fetchFilesAndUpdateCache(mockSession, mockPubStorage);

          setAuthState(AUTH_STATE.AUTHENTICATED);
        }, 2000);
      } catch (err) {
        setError(`Failed to initialize: ${err.message}`);
        setAuthState(AUTH_STATE.ERROR);
      }
    };

    initAuth();
  }, []);

  const fetchFilesAndUpdateCache = async (sess, pubStorage) => {
    try {
      const fileUrls = await getList(sess, '/pub/wiki.app/');
      const cache = {};

      for (const fileUrl of fileUrls) {
        try {
          const response = await pubStorage.get(fileUrl);
          const content = await response.text();
          const title = extractTitle(content);
          cache[fileUrl] = title;
        } catch (err) {
          console.error('Error fetching file:', err);
        }
      }

      setFileCache(cache);
    } catch (err) {
      console.error('Failed to list files:', err);
    }
  };

  const getList = async (sess, folderPath) => {
    try {
      const storage = sess.storage();
      const listRequest = storage.list(folderPath);
      const entries = await listRequest.send();
      return entries.map(entry => entry.toPubkyUrl());
    } catch (err) {
      console.error('Failed to get list:', err);
      return [];
    }
  };

  const extractTitle = (content) => {
    const firstLine = content.split('\n')[0] || '';
    return firstLine.replace(/^#\s*/, '').trim() || 'Untitled';
  };

  const navigateToViewWikiPage = async (userPk, pageId) => {
    setSelectedWikiUserId(userPk);
    setSelectedWikiPageId(pageId);
    setSelectedWikiForkUrls(await discoverForkUrls(pageId));
    setSelectedWikiContent('');
    setViewState(VIEW_STATE.VIEW_WIKI);
  };

  const discoverForkUrls = async (pageId) => {
    try {
      const follows = await getMyFollows();
      const result = [];

      // Add current user's version
      const ownPk = session.info().publicKey();
      result.push(`${ownPk}/${pageId}`);

      // Check followed users (mock - no real follows in demo)
      for (const followPk of follows) {
        const forkPath = `pubky://${followPk}/pub/wiki.app/${pageId}`;
        try {
          await publicStorage.get(forkPath);
          result.push(`${followPk}/${pageId}`);
        } catch (err) {
          // Fork doesn't exist
        }
      }

      return result;
    } catch (err) {
      console.error('Failed to discover forks:', err);
      return [];
    }
  };

  const getMyFollows = async () => {
    // Mock - return empty array for demo
    return [];
  };

  const refreshFileCache = async () => {
    if (session && publicStorage) {
      await fetchFilesAndUpdateCache(session, publicStorage);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto px-4 py-8">
        <div className="max-w-3xl mx-auto bg-white rounded-lg shadow-xl p-8">
          {/* Header */}
          <div className="text-center mb-8">
            <div className="flex justify-center mb-4">
              <img
                src="/logo.png"
                alt="Logo"
                className="w-20 h-20"
                onError={(e) => { e.target.style.display = 'none' }}
              />
            </div>
            <h1 className="text-3xl font-bold text-gray-800">{APP_NAME}</h1>
            <p className="text-sm text-gray-500 mt-2">
              (Demo Mode - Using LocalStorage)
            </p>
          </div>

          {/* Content based on auth state */}
          {authState === AUTH_STATE.INITIALIZING && (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
              <p className="text-gray-600">Initializing authentication...</p>
            </div>
          )}

          {authState === AUTH_STATE.SHOWING_QR && (
            <AuthView authUrl={authUrl} />
          )}

          {authState === AUTH_STATE.AUTHENTICATED && (
            <>
              {viewState === VIEW_STATE.WIKI_LIST && (
                <WikiList
                  fileCache={fileCache}
                  session={session}
                  onCreateNew={() => setViewState(VIEW_STATE.CREATE_WIKI)}
                  onViewWiki={navigateToViewWikiPage}
                />
              )}

              {viewState === VIEW_STATE.CREATE_WIKI && (
                <CreateWiki
                  content={editWikiContent}
                  setContent={setEditWikiContent}
                  session={session}
                  forkedFromPageId={forkedFromPageId}
                  onSave={async (newContent, filename) => {
                    await createWikiPost(session, newContent, filename);
                    await refreshFileCache();
                    setEditWikiContent('');
                    setForkedFromPageId(null);
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                  onCancel={() => {
                    setEditWikiContent('');
                    setForkedFromPageId(null);
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                />
              )}

              {viewState === VIEW_STATE.EDIT_WIKI && (
                <EditWiki
                  content={editWikiContent}
                  setContent={setEditWikiContent}
                  pageId={selectedWikiPageId}
                  session={session}
                  onUpdate={async (updatedContent) => {
                    await updateWikiPost(session, selectedWikiPageId, updatedContent);
                    await refreshFileCache();
                    setEditWikiContent('');
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                  onDelete={async () => {
                    await deleteWikiPost(session, selectedWikiPageId);
                    await refreshFileCache();
                    setEditWikiContent('');
                    setSelectedWikiPageId('');
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                  onCancel={() => {
                    setEditWikiContent('');
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                />
              )}

              {viewState === VIEW_STATE.VIEW_WIKI && (
                <ViewWiki
                  pageId={selectedWikiPageId}
                  userId={selectedWikiUserId}
                  content={selectedWikiContent}
                  setContent={setSelectedWikiContent}
                  forkUrls={selectedWikiForkUrls}
                  session={session}
                  publicStorage={publicStorage}
                  showCopyTooltip={showCopyTooltip}
                  setShowCopyTooltip={setShowCopyTooltip}
                  onNavigateToWiki={navigateToViewWikiPage}
                  onEdit={() => {
                    setEditWikiContent(selectedWikiContent);
                    setViewState(VIEW_STATE.EDIT_WIKI);
                  }}
                  onFork={() => {
                    setEditWikiContent(selectedWikiContent);
                    setForkedFromPageId(selectedWikiPageId);
                    setViewState(VIEW_STATE.CREATE_WIKI);
                  }}
                  onBack={() => {
                    setSelectedWikiPageId('');
                    setSelectedWikiContent('');
                    setViewState(VIEW_STATE.WIKI_LIST);
                  }}
                />
              )}
            </>
          )}

          {authState === AUTH_STATE.ERROR && (
            <div className="text-center py-8">
              <div className="text-red-600 mb-4">
                <svg className="w-12 h-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <h2 className="text-xl font-bold text-gray-800 mb-2">Error</h2>
              <p className="text-gray-600">{error}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Helper functions for API calls
const createWikiPost = async (session, content, filename) => {
  const path = filename
    ? `/pub/wiki.app/${filename}`
    : `/pub/wiki.app/${crypto.randomUUID()}`;

  await session.storage().put(path, content);
  console.log('Created post at path:', path);
  return path;
};

const updateWikiPost = async (session, pageId, content) => {
  const path = `/pub/wiki.app/${pageId}`;
  await session.storage().put(path, content);
  console.log('Updated post at path:', path);
};

const deleteWikiPost = async (session, pageId) => {
  const path = `/pub/wiki.app/${pageId}`;
  await session.storage().delete(path);
  console.log('Deleted post at path:', path);
};

export default App;
