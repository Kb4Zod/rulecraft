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
            btn.querySelector('.bookmark-icon').textContent = '★';
            btn.querySelector('.bookmark-label').textContent = 'Bookmarked';
        } else {
            btn.classList.remove('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '☆';
            btn.querySelector('.bookmark-label').textContent = 'Bookmark';
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
        alert('No bookmarks yet. Click the bookmark button on any rule to save it.');
        return;
    }

    // Create modal
    const modal = document.createElement('div');
    modal.className = 'bookmarks-modal';
    modal.innerHTML = `
        <div class="bookmarks-content">
            <div class="bookmarks-header">
                <h2>Bookmarked Rules</h2>
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

    // Add modal styles
    const style = document.createElement('style');
    style.textContent = `
        .bookmarks-modal {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0, 0, 0, 0.8);
            display: flex;
            justify-content: center;
            align-items: center;
            z-index: 1000;
        }
        .bookmarks-content {
            background: var(--color-surface);
            border-radius: var(--radius);
            padding: 1.5rem;
            max-width: 500px;
            width: 90%;
            max-height: 80vh;
            overflow-y: auto;
        }
        .bookmarks-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
        }
        .close-btn {
            background: none;
            border: none;
            font-size: 1.5rem;
            cursor: pointer;
            color: var(--color-text-muted);
        }
        .bookmark-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.5rem;
            border-bottom: 1px solid var(--color-border);
        }
        .bookmark-item a {
            color: var(--color-text);
            text-decoration: none;
        }
        .bookmark-item a:hover {
            color: var(--color-primary);
        }
        .remove-btn {
            background: transparent;
            border: 1px solid var(--color-secondary);
            color: var(--color-secondary);
            padding: 0.25rem 0.5rem;
            border-radius: 4px;
            cursor: pointer;
        }
        .bookmarks-actions {
            display: flex;
            gap: 0.5rem;
            margin-top: 1rem;
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
    if (confirm('Are you sure you want to clear all bookmarks?')) {
        saveBookmarks({});
        closeBookmarksModal();
        // Update all bookmark buttons on page
        document.querySelectorAll('.bookmark-btn').forEach(btn => {
            btn.classList.remove('bookmarked');
            btn.querySelector('.bookmark-icon').textContent = '☆';
            btn.querySelector('.bookmark-label').textContent = 'Bookmark';
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
            btn.querySelector('.bookmark-icon').textContent = '★';
            btn.querySelector('.bookmark-label').textContent = 'Bookmarked';
        }
    });
});
