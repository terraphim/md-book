import { PagefindSearch } from '../js/pagefind-search.js';

/**
 * Search Modal Component
 * Provides a modal interface for search functionality
 */

class SearchModal extends HTMLElement {
    constructor() {
        super();
        this.isOpen = false;
        this.search = null;
        this.currentResults = [];
        this.selectedIndex = -1;

        // Bind methods
        this.handleKeydown = this.handleKeydown.bind(this);
        this.handleClickOutside = this.handleClickOutside.bind(this);
        this.handleSearchResults = this.handleSearchResults.bind(this);
        this.handleInput = this.handleInput.bind(this);
    }

    connectedCallback() {
        this.render();
        this.setupEventListeners();
        this.initializeSearch();
    }

    disconnectedCallback() {
        this.removeEventListeners();
        if (this.search) {
            this.search.destroy();
        }
        this.input?.removeEventListener('input', this.handleInput);
    }

    render() {
        this.innerHTML = `
            <div class="search-modal-overlay" style="display: none;">
                <div class="search-modal" role="dialog" aria-modal="true" aria-label="Search documentation">
                    <div class="search-modal-header">
                        <div class="search-input-container">
                            <wa-input
                                class="search-input"
                                placeholder="Search documentation..."
                                size="large"
                                clearable
                                aria-controls="search-results"
                                autofocus>
                                <wa-icon name="magnifying-glass" slot="start"></wa-icon>
                            </wa-input>
                        </div>
                        <wa-button class="search-close-btn" variant="text" size="small" aria-label="Close search">
                            <wa-icon name="xmark"></wa-icon>
                        </wa-button>
                    </div>

                    <div class="search-results-container">
                        <div class="search-results" id="search-results" role="listbox" aria-label="Search results"></div>
                        <div class="search-footer">
                            <div class="search-shortcuts">
                                <span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span>
                                <span><kbd>Enter</kbd> Select</span>
                                <span><kbd>Esc</kbd> Close</span>
                            </div>
                        </div>
                    </div>

                    <div class="search-loading" style="display: none;">
                        <wa-spinner></wa-spinner>
                        <span>Searching...</span>
                    </div>

                    <div class="search-empty" style="display: none;">
                        <wa-icon name="magnifying-glass" class="search-empty-icon"></wa-icon>
                        <p>No results found</p>
                        <p class="search-empty-subtitle">Try adjusting your search terms</p>
                    </div>
                </div>
            </div>
        `;

        this.setupModalElements();
    }

    setupModalElements() {
        this.overlay = this.querySelector('.search-modal-overlay');
        this.modal = this.querySelector('.search-modal');
        this.input = this.querySelector('.search-input');
        this.closeBtn = this.querySelector('.search-close-btn');
        this.resultsContainer = this.querySelector('.search-results');
        this.loadingElement = this.querySelector('.search-loading');
        this.emptyElement = this.querySelector('.search-empty');
    }

    async initializeSearch() {
        try {
            this.search = new PagefindSearch({
                debounceDelay: 150,
                minQueryLength: 2,
                maxResults: 10
            });

            // Handle URL parameters
            const initialQuery = this.search.handleUrlParams();
            if (initialQuery) {
                this.input.value = initialQuery;
                this.performSearch(initialQuery);
            }
        } catch (error) {
            console.error('Failed to initialize search:', error);
        }
    }

    setupEventListeners() {
        // Keyboard shortcuts
        document.addEventListener('keydown', this.handleKeydown);

        // Modal events
        this.closeBtn?.addEventListener('click', () => this.close());
        this.overlay?.addEventListener('click', this.handleClickOutside);

        // Search input events
        this.input?.addEventListener('input', this.handleInput);

        this.input?.addEventListener('keydown', (e) => {
            if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                e.preventDefault();
                this.navigateResults(e.key === 'ArrowDown' ? 1 : -1);
            } else if (e.key === 'Enter') {
                e.preventDefault();
                this.selectCurrentResult();
            }
        });
    }

    removeEventListeners() {
        document.removeEventListener('keydown', this.handleKeydown);
    }

    handleInput(e) {
        const source = e.composedPath()[0];
        const query = (source.value || this.input.value || '').trim();
        this.performSearch(query);
    }

    handleKeydown(e) {
        // Open search modal with '/' or 'Cmd+K'
        if (e.key === '/' || (e.key === 'k' && (e.metaKey || e.ctrlKey))) {
            const target = e.target;
            if (target instanceof HTMLElement &&
                (target.isContentEditable ||
                 target.matches('input, textarea, select, sl-input, wa-input'))) return;
            e.preventDefault();
            this.open();
            return;
        }

        // Close modal with Escape
        if (e.key === 'Escape' && this.isOpen) {
            e.preventDefault();
            this.close();
            return;
        }
    }

    handleClickOutside(e) {
        if (e.target === this.overlay) {
            this.close();
        }
    }

    async performSearch(query) {
        if (!this.search) return;

        // Update URL
        this.search.updateUrl(query);

        if (!query || query.length < this.search.options.minQueryLength) {
            this.showEmpty();
            return;
        }

        this.showLoading();

        try {
            await this.search.search(query, this.handleSearchResults);
        } catch (error) {
            console.error('Search error:', error);
            this.showEmpty();
        }
    }

    handleSearchResults(searchData, error) {
        this.hideLoading();

        if (error) {
            this.showEmpty();
            return;
        }

        this.currentResults = searchData.results || [];

        if (this.currentResults.length === 0) {
            this.showEmpty();
            return;
        }

        this.renderResults(searchData);
    }

    renderResults(searchData) {
        const { query, results, totalResults } = searchData;

        this.resultsContainer.innerHTML = '';
        this.selectedIndex = -1;

        results.forEach((result, index) => {
            const resultElement = this.createResultElement(result, query, index);
            this.resultsContainer.appendChild(resultElement);
        });

        this.emptyElement.style.display = 'none';
        this.resultsContainer.parentElement.style.display = 'block';
    }

    createResultElement(result, query, index) {
        const element = document.createElement('div');
        element.className = 'search-result-item';
        element.setAttribute('data-index', index);
        element.setAttribute('role', 'option');
        element.setAttribute('aria-selected', 'false');

        const content = document.createElement('div');
        content.className = 'search-result-content';
        const title = document.createElement('h3');
        title.className = 'search-result-title';
        title.textContent = result.title;
        const excerpt = document.createElement('p');
        excerpt.className = 'search-result-excerpt';
        // Pagefind excerpts are entity-encoded before their <mark> elements.
        excerpt.innerHTML = result.excerpt;
        const url = document.createElement('span');
        url.className = 'search-result-url';
        url.textContent = result.url;
        content.append(title, excerpt, url);
        const action = document.createElement('div');
        action.className = 'search-result-action';
        const icon = document.createElement('wa-icon');
        icon.name = 'arrow-right';
        action.appendChild(icon);
        element.append(content, action);

        element.addEventListener('click', () => {
            this.selectResult(result);
        });

        element.addEventListener('mouseenter', () => {
            this.setSelectedIndex(index);
        });

        return element;
    }

    navigateResults(direction) {
        if (this.currentResults.length === 0) return;

        const newIndex = this.selectedIndex + direction;

        if (newIndex >= 0 && newIndex < this.currentResults.length) {
            this.setSelectedIndex(newIndex);
        } else if (direction > 0 && this.selectedIndex === this.currentResults.length - 1) {
            this.setSelectedIndex(0);
        } else if (direction < 0 && this.selectedIndex === 0) {
            this.setSelectedIndex(this.currentResults.length - 1);
        }
    }

    setSelectedIndex(index) {
        // Remove previous selection
        const previousSelected = this.resultsContainer.querySelector('.selected');
        if (previousSelected) {
            previousSelected.classList.remove('selected');
            previousSelected.setAttribute('aria-selected', 'false');
        }

        this.selectedIndex = index;

        // Add selection to current item
        const currentItem = this.resultsContainer.querySelector(`[data-index="${index}"]`);
        if (currentItem) {
            currentItem.classList.add('selected');
            currentItem.setAttribute('aria-selected', 'true');
            currentItem.scrollIntoView({ block: 'nearest' });
        }
    }

    selectCurrentResult() {
        if (this.selectedIndex >= 0 && this.currentResults[this.selectedIndex]) {
            this.selectResult(this.currentResults[this.selectedIndex]);
        }
    }

    selectResult(result) {
        // Navigate to the result
        window.location.href = result.url;
    }

    showLoading() {
        this.loadingElement.style.display = 'flex';
        this.emptyElement.style.display = 'none';
        this.resultsContainer.parentElement.style.display = 'none';
    }

    hideLoading() {
        this.loadingElement.style.display = 'none';
    }

    showEmpty() {
        this.hideLoading();
        this.emptyElement.style.display = 'flex';
        this.resultsContainer.parentElement.style.display = 'none';
        this.currentResults = [];
        this.selectedIndex = -1;
    }

    open() {
        this.previouslyFocused = document.activeElement;
        this.isOpen = true;
        this.overlay.style.display = 'flex';

        // Focus input after modal opens
        requestAnimationFrame(() => {
            this.input?.focus();
        });

        // Prevent body scroll
        document.body.style.overflow = 'hidden';
    }

    close() {
        this.isOpen = false;
        this.overlay.style.display = 'none';

        // Restore body scroll
        document.body.style.overflow = '';

        // Clear selection
        this.selectedIndex = -1;
        this.currentResults = [];
        const selected = this.resultsContainer.querySelector('.selected');
        selected?.classList.remove('selected');
        selected?.setAttribute('aria-selected', 'false');
        this.dispatchEvent(new CustomEvent('close'));
        this.previouslyFocused?.focus?.();
    }

    // Public API
    triggerSearch(query) {
        this.input.value = query;
        this.performSearch(query);
        this.open();
    }
}

// Define the custom element
customElements.define('search-modal', SearchModal);
