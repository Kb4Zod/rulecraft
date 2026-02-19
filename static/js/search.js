/**
 * Rulecraft Search Module
 * Handles fuzzy search with autocomplete suggestions
 */

let debounceTimer;
let currentFocus = -1;

/**
 * Initialize search autocomplete on all search inputs
 */
function initSearchAutocomplete() {
    const searchInputs = document.querySelectorAll('.search-input, #search-input');

    searchInputs.forEach(input => {
        // Create suggestions container
        const wrapper = document.createElement('div');
        wrapper.className = 'search-wrapper';
        input.parentNode.insertBefore(wrapper, input);
        wrapper.appendChild(input);

        const suggestionsDiv = document.createElement('div');
        suggestionsDiv.className = 'search-suggestions';
        suggestionsDiv.id = 'search-suggestions-' + Math.random().toString(36).substr(2, 9);
        wrapper.appendChild(suggestionsDiv);

        // Input event for typing
        input.addEventListener('input', (e) => {
            clearTimeout(debounceTimer);
            const query = e.target.value.trim();

            if (query.length < 2) {
                closeSuggestions(suggestionsDiv);
                return;
            }

            // Debounce API calls
            debounceTimer = setTimeout(() => {
                fetchSuggestions(query, suggestionsDiv, input);
            }, 150);
        });

        // Keyboard navigation
        input.addEventListener('keydown', (e) => {
            const items = suggestionsDiv.querySelectorAll('.suggestion-item');

            if (e.key === 'ArrowDown') {
                e.preventDefault();
                currentFocus++;
                setActiveSuggestion(items);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                currentFocus--;
                setActiveSuggestion(items);
            } else if (e.key === 'Enter') {
                if (currentFocus > -1 && items[currentFocus]) {
                    e.preventDefault();
                    items[currentFocus].click();
                }
            } else if (e.key === 'Escape') {
                closeSuggestions(suggestionsDiv);
            }
        });

        // Close on click outside
        document.addEventListener('click', (e) => {
            if (!wrapper.contains(e.target)) {
                closeSuggestions(suggestionsDiv);
            }
        });

        // Close on focus out
        input.addEventListener('blur', () => {
            // Delay to allow click on suggestion
            setTimeout(() => closeSuggestions(suggestionsDiv), 200);
        });
    });
}

/**
 * Fetch suggestions from API
 */
async function fetchSuggestions(query, container, input) {
    try {
        const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
        const suggestions = await response.json();

        displaySuggestions(suggestions, container, input, query);
    } catch (error) {
        console.error('Search error:', error);
        closeSuggestions(container);
    }
}

/**
 * Display suggestions in dropdown
 */
function displaySuggestions(suggestions, container, input, query) {
    currentFocus = -1;
    container.innerHTML = '';

    if (suggestions.length === 0) {
        container.innerHTML = '<div class="no-results">No rules found</div>';
        container.classList.add('active');
        return;
    }

    suggestions.forEach((suggestion, index) => {
        const item = document.createElement('div');
        item.className = 'suggestion-item';
        item.dataset.index = index;

        // Highlight matching text
        const highlightedTitle = highlightMatch(suggestion.title, query);
        const highlightedExcerpt = highlightMatch(suggestion.excerpt, query);

        item.innerHTML = `
            <div class="suggestion-title">${highlightedTitle}</div>
            <div class="suggestion-meta">
                <span class="suggestion-category">${suggestion.category}</span>
            </div>
            <div class="suggestion-excerpt">${highlightedExcerpt}</div>
        `;

        item.addEventListener('click', () => {
            window.location.href = `/rules/${suggestion.id}`;
        });

        item.addEventListener('mouseenter', () => {
            currentFocus = index;
            setActiveSuggestion(container.querySelectorAll('.suggestion-item'));
        });

        container.appendChild(item);
    });

    // Add "View all results" link
    const viewAll = document.createElement('div');
    viewAll.className = 'suggestion-view-all';
    viewAll.innerHTML = `<a href="/search?q=${encodeURIComponent(query)}">View all results →</a>`;
    container.appendChild(viewAll);

    container.classList.add('active');
}

/**
 * Highlight matching text in string
 */
function highlightMatch(text, query) {
    if (!query) return text;

    const regex = new RegExp(`(${escapeRegex(query)})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
}

/**
 * Escape special regex characters
 */
function escapeRegex(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Set active suggestion for keyboard navigation
 */
function setActiveSuggestion(items) {
    if (!items.length) return;

    // Remove active class from all
    items.forEach(item => item.classList.remove('active'));

    // Wrap around
    if (currentFocus >= items.length) currentFocus = 0;
    if (currentFocus < 0) currentFocus = items.length - 1;

    // Add active class
    items[currentFocus].classList.add('active');
    items[currentFocus].scrollIntoView({ block: 'nearest' });
}

/**
 * Close suggestions dropdown
 */
function closeSuggestions(container) {
    container.classList.remove('active');
    container.innerHTML = '';
    currentFocus = -1;
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', initSearchAutocomplete);
