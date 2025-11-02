import { useState, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';

function ViewWiki({
  pageId,
  userId,
  content,
  setContent,
  forkUrls,
  session,
  publicStorage,
  showCopyTooltip,
  setShowCopyTooltip,
  onNavigateToWiki,
  onEdit,
  onFork,
  onBack
}) {
  const [showForks, setShowForks] = useState(false);
  const [showDetails, setShowDetails] = useState(false);

  const ownPk = session?.info()?.publicKey() || '';
  const isOwnPage = userId === ownPk;

  // Fetch content when component mounts or pageId/userId changes
  useEffect(() => {
    const fetchContent = async () => {
      if (!content && pageId && userId) {
        try {
          const path = `pubky://${userId}/pub/wiki.app/${pageId}`;
          const response = await publicStorage.get(path);
          const text = await response.text();
          setContent(text);
        } catch (err) {
          console.error('Error fetching content:', err);
          setContent(`Error loading content: ${err.message}`);
        }
      }
    };

    fetchContent();
  }, [pageId, userId, content, publicStorage, setContent]);

  const handleShareLink = () => {
    const link = `[link](${userId}/${pageId})`;
    navigator.clipboard.writeText(link);
    setShowCopyTooltip(true);
    setTimeout(() => setShowCopyTooltip(false), 2000);
  };

  const handleLinkClick = (href) => {
    // Check if it's a wiki link (user_id/page_id format)
    const parts = href.split('/');
    if (parts.length === 2 && !href.startsWith('http')) {
      const [userPk, pageId] = parts;
      onNavigateToWiki(userPk, pageId);
      return false;
    }
    return true;
  };

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-800 mb-6">View Wiki Post</h2>

      {/* Collapsible Page Details */}
      <div className="mb-4">
        <button
          onClick={() => setShowDetails(!showDetails)}
          className="text-blue-600 hover:text-blue-800 font-semibold flex items-center gap-2"
        >
          📋 Page Details
          <span className="text-sm">{showDetails ? '▼' : '▶'}</span>
        </button>
        {showDetails && (
          <div className="mt-2 p-3 bg-gray-50 rounded border border-gray-200 text-sm">
            <p className="font-mono text-gray-700">Page ID: {pageId}</p>
            <p className="font-mono text-gray-700 mt-1">User ID: {userId}</p>
          </div>
        )}
      </div>

      {/* Collapsible Forks */}
      <div className="mb-4">
        <button
          onClick={() => setShowForks(!showForks)}
          className="text-blue-600 hover:text-blue-800 font-semibold flex items-center gap-2"
        >
          🔀 Available Forks ({forkUrls.length})
          <span className="text-sm">{showForks ? '▼' : '▶'}</span>
        </button>
        {showForks && (
          <div className="mt-2 space-y-2">
            {forkUrls.map((forkUrl) => {
              const [userPk, forkPageId] = forkUrl.split('/');
              const isCurrent = userPk === userId;
              return (
                <button
                  key={forkUrl}
                  onClick={() => !isCurrent && onNavigateToWiki(userPk, forkPageId)}
                  className={`block w-full text-left p-2 rounded ${
                    isCurrent
                      ? 'bg-blue-100 text-blue-800 font-semibold'
                      : 'bg-gray-50 hover:bg-gray-100 text-gray-700'
                  }`}
                  disabled={isCurrent}
                >
                  Fork: {userPk.substring(0, 16)}... {isCurrent && '(current)'}
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* Share Link Button */}
      <div className="mb-4 relative">
        <button
          onClick={handleShareLink}
          className="bg-indigo-600 hover:bg-indigo-700 text-white font-semibold py-2 px-4 rounded-lg shadow-md transition duration-200"
        >
          🔗 Share Page Link
        </button>
        {showCopyTooltip && (
          <span className="absolute left-0 top-full mt-1 bg-gray-800 text-white text-sm px-2 py-1 rounded">
            Copied!
          </span>
        )}
      </div>

      {/* Content Display */}
      <div className="border-t border-gray-300 pt-4 mb-6">
        <div className="bg-gray-50 p-6 rounded-lg">
          <ReactMarkdown
            components={{
              a: ({ node, href, children, ...props }) => (
                <a
                  href={href}
                  onClick={(e) => {
                    if (!handleLinkClick(href)) {
                      e.preventDefault();
                    }
                  }}
                  {...props}
                  className="text-blue-600 hover:text-blue-800 underline cursor-pointer"
                >
                  {children}
                </a>
              ),
              img: ({ node, src, alt, ...props }) => (
                <img
                  src={src}
                  alt={alt}
                  {...props}
                  className="max-w-full h-auto"
                  style={{ maxWidth: '512px' }}
                />
              )
            }}
          >
            {content || 'Loading...'}
          </ReactMarkdown>
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-3">
        {isOwnPage && (
          <button
            onClick={onEdit}
            className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
          >
            ✏ Edit
          </button>
        )}
        {!isOwnPage && (
          <button
            onClick={onFork}
            className="bg-purple-600 hover:bg-purple-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
          >
            🍴 Fork
          </button>
        )}
        <button
          onClick={onBack}
          className="bg-gray-500 hover:bg-gray-600 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
        >
          ← Back
        </button>
      </div>
    </div>
  );
}

export default ViewWiki;
