function WikiList({ fileCache, session, onCreateNew, onViewWiki }) {
  const ownPk = session?.info()?.publicKey() || '';

  return (
    <div>
      <div className="mb-6">
        <button
          onClick={onCreateNew}
          className="w-full sm:w-auto bg-blue-600 hover:bg-blue-700 text-white font-semibold py-3 px-6 rounded-lg shadow-md transition duration-200"
        >
          ✨ Create New Wiki Page
        </button>
      </div>

      <h2 className="text-2xl font-bold text-gray-800 mb-4">My Wiki Posts</h2>

      <div className="space-y-2">
        {Object.keys(fileCache).length === 0 ? (
          <p className="text-gray-500 italic py-4">
            No wiki posts yet. Create your first one!
          </p>
        ) : (
          Object.entries(fileCache).map(([fileUrl, fileTitle]) => {
            const fileName = fileUrl.split('/').pop();
            return (
              <div
                key={fileUrl}
                className="flex items-center gap-3 p-3 bg-gray-50 hover:bg-gray-100 rounded-lg transition duration-150 cursor-pointer"
                onClick={() => onViewWiki(ownPk, fileName)}
              >
                <code className="text-sm bg-white px-2 py-1 rounded border border-gray-300">
                  {fileName}
                </code>
                <span className="font-semibold text-gray-700">{fileTitle}</span>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}

export default WikiList;
