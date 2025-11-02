function EditWiki({ content, setContent, pageId, onUpdate, onDelete, onCancel }) {
  const handleUpdate = () => {
    onUpdate(content);
  };

  const handleDelete = () => {
    if (window.confirm('Are you sure you want to delete this wiki page?')) {
      onDelete();
    }
  };

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-800 mb-6">Edit Wiki Page</h2>

      <div className="mb-6">
        <label className="block text-gray-700 font-semibold mb-2">
          Content:
        </label>
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="w-full h-96 p-4 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
        />
      </div>

      <div className="flex gap-3">
        <button
          onClick={handleUpdate}
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
        >
          ✓ Update
        </button>
        <button
          onClick={handleDelete}
          className="bg-red-600 hover:bg-red-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
        >
          🗑 Delete
        </button>
        <button
          onClick={onCancel}
          className="bg-gray-500 hover:bg-gray-600 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

export default EditWiki;
