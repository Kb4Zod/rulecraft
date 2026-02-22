/**
 * Rulecraft Bookmarks Module
 * Handles local storage for rule bookmarks
 */

const STORAGE_KEY = 'rulecraft_bookmarks';

/**
 * Get all bookmarks from local storage
 * @returns {Object} Map of rule ID to bookmark data
 */
function getBookmarks() {
    try {
        const data = localStorage.getItem(STORAGE_KEY);
        return data ? JSON.parse(data) : {};
    } catch (e) {
        console.error('Error reading bookmarks:', e);
        return {};
    }
}

/**
 * Save bookmarks to local storage
 * @param {Object} bookmarks - Map of rule ID to bookmark data
 */
function saveBookmarks(bookmarks) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(bookmarks));
    } catch (e) {
        console.error('Error saving bookmarks:', e);
    }
}

/**
 * Check if a rule is bookmarked
 * @param {string} ruleId - The rule ID
 * @returns {boolean}
 */
function isBookmarked(ruleId) {
    const bookmarks = getBookmarks();
    return ruleId in bookmarks;
}

/**
 * Toggle bookmark for a rule
 * @param {string} ruleId - The rule ID
 * @param {string} title - The rule title
 */
function toggleBookmark(ruleId, title) {
    const bookmarks = getBookmarks();

    if (ruleId in bookmarks) {
        delete bookmarks[ruleId];
        updateBookmarkButton(ruleId, false);
    } else {
        bookmarks[ruleId] = {
            title: title,
            addedAt: new Date().toISOString()
        };
        updateBookmarkButton(ruleId, true);
    }

    saveBookmarks(bookmarks);
}

/**
 * Update bookmark button state
 * @param {string} ruleId - The rule ID
 * @param {boolean} bookmarked - Whether the rule is bookmarked
 */
function updateBookmarkButton(ruleId, bookmarked) {
    const buttons = document.querySelectorAll(`.bookmark-btn[data-rule-id="${ruleId}"]`);
    buttons.forEach(btn => {
        if (bookmarked) {
            btn.classList.add('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '\u2605';
            btn.querySelector('.bookmark-label').textContent = 'Marked';
        } else {
            btn.classList.remove('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '\u2606';
            btn.querySelector('.bookmark-label').textContent = 'Mark';
        }
    });
}

/**
 * Show bookmarks modal/panel
 */
function showBookmarks() {
    const bookmarks = getBookmarks();
    const entries = Object.entries(bookmarks);

    if (entries.length === 0) {
        alert('No marks yet. Click the mark button on any rule to save it.');
        return;
    }

    // Create modal
    const modal = document.createElement('div');
    modal.className = 'bookmarks-modal';
    modal.innerHTML = `
        <div class="bookmarks-content">
            <div class="bookmarks-header">
                <h2>Thy Marked Passages</h2>
                <button onclick="closeBookmarksModal()" class="close-btn">&times;</button>
            </div>
            <div class="bookmarks-list">
                ${entries.map(([id, data]) => `
                    <div class="bookmark-item">
                        <a href="/rules/${id}">${data.title}</a>
                        <button onclick="removeBookmark('${id}')" class="remove-btn">Remove</button>
                    </div>
                `).join('')}
            </div>
            <div class="bookmarks-actions">
                <button onclick="exportBookmarks()" class="btn btn-secondary">Export</button>
                <button onclick="importBookmarks()" class="btn btn-secondary">Import</button>
                <button onclick="clearAllBookmarks()" class="btn btn-secondary">Clear All</button>
            </div>
        </div>
    `;

    // Add modal styles with grimoire theme
    const style = document.createElement('style');
    style.textContent = `
        .bookmarks-modal {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(26, 15, 10, 0.9);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        }
        .bookmarks-content {
            background: #f4e4c1;
            border: 3px solid #c9a227;
            border-radius: 16px;
            padding: 2rem;
            max-width: 500px;
            width: 90%;
            max-height: 80vh;
            overflow-y: auto;
            box-shadow: 0 8px 30px rgba(45, 24, 16, 0.4);
        }
        .bookmarks-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1.5rem;
            padding-bottom: 1rem;
            border-bottom: 2px solid #704214;
        }
        .bookmarks-header h2 {
            font-family: 'Cinzel Decorative', serif;
            color: #722f37;
            font-size: 1.5rem;
            letter-spacing: 1px;
        }
        .close-btn {
            background: none;
            border: none;
            font-size: 2rem;
            cursor: pointer;
            color: #722f37;
            transition: color 0.3s;
        }
        .close-btn:hover {
            color: #501c22;
        }
        .bookmark-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem 1rem;
            background: #faf3e3;
            border: 1px solid #e8d5a3;
            border-left: 3px solid #722f37;
            border-radius: 8px;
            margin-bottom: 0.5rem;
            transition: all 0.3s;
        }
        .bookmark-item:hover {
            background: #f4e4c1;
            border-color: #722f37;
        }
        .bookmark-item a {
            color: #2d1810;
            text-decoration: none;
            font-family: 'Crimson Text', serif;
            font-size: 1rem;
        }
        .bookmark-item a:hover {
            color: #722f37;
        }
        .remove-btn {
            background: transparent;
            border: 1px solid #722f37;
            color: #722f37;
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            cursor: pointer;
            font-family: 'Cinzel', serif;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            transition: all 0.3s;
        }
        .remove-btn:hover {
            background: #722f37;
            color: #f4e4c1;
        }
        .bookmarks-actions {
            display: flex;
            gap: 0.5rem;
            margin-top: 1.5rem;
            padding-top: 1rem;
            border-top: 2px solid #704214;
        }
        .bookmarks-actions .btn {
            flex: 1;
            padding: 0.5rem 1rem;
            border-radius: 20px;
            font-family: 'Cinzel', serif;
            font-size: 0.8rem;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            cursor: pointer;
            transition: all 0.3s;
        }
        .bookmarks-actions .btn-secondary {
            background: #e8d5a3;
            border: 1px solid #704214;
            color: #2d1810;
        }
        .bookmarks-actions .btn-secondary:hover {
            background: #722f37;
            border-color: #722f37;
            color: #f4e4c1;
        }
    `;

    document.head.appendChild(style);
    document.body.appendChild(modal);

    // Close on background click
    modal.addEventListener('click', (e) => {
        if (e.target === modal) {
            closeBookmarksModal();
        }
    });
}

/**
 * Close bookmarks modal
 */
function closeBookmarksModal() {
    const modal = document.querySelector('.bookmarks-modal');
    if (modal) {
        modal.remove();
    }
}

/**
 * Remove a bookmark
 * @param {string} ruleId - The rule ID
 */
function removeBookmark(ruleId) {
    const bookmarks = getBookmarks();
    delete bookmarks[ruleId];
    saveBookmarks(bookmarks);
    updateBookmarkButton(ruleId, false);
    closeBookmarksModal();
    showBookmarks();
}

/**
 * Export bookmarks as JSON
 */
function exportBookmarks() {
    const bookmarks = getBookmarks();
    const data = JSON.stringify(bookmarks, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'rulecraft-bookmarks.json';
    a.click();
    URL.revokeObjectURL(url);
}

/**
 * Import bookmarks from JSON file
 */
function importBookmarks() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
        const file = e.target.files[0];
        if (file) {
            const reader = new FileReader();
            reader.onload = (e) => {
                try {
                    const imported = JSON.parse(e.target.result);
                    const current = getBookmarks();
                    const merged = { ...current, ...imported };
                    saveBookmarks(merged);
                    alert('Bookmarks imported successfully!');
                    closeBookmarksModal();
                    showBookmarks();
                } catch (err) {
                    alert('Error importing bookmarks: Invalid JSON file');
                }
            };
            reader.readAsText(file);
        }
    };
    input.click();
}

/**
 * Clear all bookmarks
 */
function clearAllBookmarks() {
    if (confirm('Art thou certain thou wish to clear all marks?')) {
        saveBookmarks({});
        closeBookmarksModal();
        // Update all bookmark buttons on page
        document.querySelectorAll('.bookmark-btn').forEach(btn => {
            btn.classList.remove('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '\u2606';
            btn.querySelector('.bookmark-label').textContent = 'Mark';
        });
    }
}

// Initialize bookmark buttons on page load
document.addEventListener('DOMContentLoaded', () => {
    const bookmarks = getBookmarks();
    document.querySelectorAll('.bookmark-btn').forEach(btn => {
        const ruleId = btn.dataset.ruleId;
        if (ruleId && isBookmarked(ruleId)) {
            btn.classList.add('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '\u2605';
            btn.querySelector('.bookmark-label').textContent = 'Marked';
        }
    });
});
