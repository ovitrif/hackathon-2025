function CreateWiki({ content, setContent, onSave, onCancel, forkedFromPageId }) {
  const handleSave = () => {
    onSave(content, forkedFromPageId);
  };

  return (
    <div>
      <h2 className="text-2xl font-bold text-gray-800 mb-6">
        {forkedFromPageId ? '🍴 Fork Wiki Page' : 'Create New Wiki Page'}
      </h2>

      <div className="mb-6">
        <label className="block text-gray-700 font-semibold mb-2">
          Content:
        </label>
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="w-full h-96 p-4 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
          placeholder="# Your Wiki Title

Start writing your wiki content here using Markdown...

## Features
- Bullet points
- **Bold text**
- *Italic text*
- [Links](url)
"
        />
      </div>

      <div className="flex gap-3">
        <button
          onClick={handleSave}
          className="bg-green-600 hover:bg-green-700 text-white font-semibold py-2 px-6 rounded-lg shadow-md transition duration-200"
        >
          💾 Save
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

export default CreateWiki;
