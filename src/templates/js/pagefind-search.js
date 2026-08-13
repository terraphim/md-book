/**
 * Pagefind Search Wrapper
 * Provides a clean API for interacting with Pagefind search functionality
 */

class PagefindSearch {
    constructor(options = {}) {
        this.options = {
            bundlePath: new URL('../pagefind/', import.meta.url).href,
            baseUrl: new URL('../', import.meta.url).href,
            debounceDelay: 150,
            minQueryLength: 2,
            maxResults: 20,
            ...options
        };
        
        this.pagefind = null;
        this.isInitialized = false;
        this.searchGeneration = 0;
        this.searchHistory = this.loadSearchHistory();
    }

    /**
     * Initialize Pagefind
     */
    async init() {
        if (this.isInitialized) return;
        
        try {
            this.pagefind = await import(new URL('pagefind.js', this.options.bundlePath).href);
            await this.pagefind.options({
                basePath: this.options.bundlePath,
                baseUrl: this.options.baseUrl
            });
            await this.pagefind.init();
            this.isInitialized = true;
            console.log('Pagefind initialized successfully');
        } catch (error) {
            console.error('Failed to initialize Pagefind:', error);
            throw new Error(`Pagefind initialization failed: ${error.message}`);
        }
    }

    /**
     * Perform search with debouncing
     */
    async search(query, callback) {
        const normalizedQuery = query.trim();
        const generation = ++this.searchGeneration;

        if (normalizedQuery.length < this.options.minQueryLength) {
            callback({ query: normalizedQuery, results: [], totalResults: 0 });
            return;
        }

        try {
            if (!this.isInitialized) await this.init();
            const searchResult = await this.pagefind.debouncedSearch(
                normalizedQuery,
                {},
                this.options.debounceDelay
            );
            if (searchResult === null || generation !== this.searchGeneration) return;

            const results = await this.loadResults(normalizedQuery, searchResult);
            if (generation !== this.searchGeneration) return;
            this.addToSearchHistory(normalizedQuery);
            callback(results);
        } catch (error) {
            if (generation !== this.searchGeneration) return;
            console.error('Search failed:', error);
            callback({ query: normalizedQuery, results: [], totalResults: 0 }, error);
        }
    }

    /**
     * Perform immediate search without debouncing
     */
    async performSearch(query) {
        if (!this.isInitialized) {
            await this.init();
        }

        const searchResult = await this.pagefind.search(query);
        return this.loadResults(query, searchResult);
    }

    async loadResults(query, searchResult) {
        const results = await Promise.all(
            searchResult.results
                .slice(0, this.options.maxResults)
                .map(async (result) => {
                    const data = await result.data();
                    return {
                        url: data.url,
                        title: data.meta.title || 'Untitled',
                        excerpt: data.excerpt,
                        content: data.content,
                        score: result.score,
                        subResults: data.sub_results || []
                    };
                })
        );

        return {
            query,
            results,
            totalResults: searchResult.results.length,
            unfilteredResultCount: searchResult.unfilteredResultCount
        };
    }

    /**
     * Handle URL parameters for search
     */
    handleUrlParams() {
        const urlParams = new URLSearchParams(window.location.search);
        const searchQuery = urlParams.get('q');
        
        return searchQuery || null;
    }

    /**
     * Update URL with search query
     */
    updateUrl(query) {
        const url = new URL(window.location);
        if (query && query.trim()) {
            url.searchParams.set('q', query.trim());
        } else {
            url.searchParams.delete('q');
        }
        
        // Update URL without page reload
        window.history.replaceState({}, '', url.toString());
    }

    /**
     * Load search history from localStorage
     */
    loadSearchHistory() {
        try {
            const history = localStorage.getItem('pagefind-search-history');
            return history ? JSON.parse(history) : [];
        } catch (error) {
            console.warn('Failed to load search history:', error);
            return [];
        }
    }

    /**
     * Save search history to localStorage
     */
    saveSearchHistory() {
        try {
            localStorage.setItem('pagefind-search-history', JSON.stringify(this.searchHistory));
        } catch (error) {
            console.warn('Failed to save search history:', error);
        }
    }

    /**
     * Add query to search history
     */
    addToSearchHistory(query) {
        if (!query || query.length < this.options.minQueryLength) return;
        
        // Remove duplicates and add to beginning
        this.searchHistory = this.searchHistory.filter(item => item !== query);
        this.searchHistory.unshift(query);
        
        // Limit history size
        if (this.searchHistory.length > 10) {
            this.searchHistory = this.searchHistory.slice(0, 10);
        }
        
        this.saveSearchHistory();
    }

    /**
     * Get search history
     */
    getSearchHistory() {
        return [...this.searchHistory];
    }

    /**
     * Clear search history
     */
    clearSearchHistory() {
        this.searchHistory = [];
        this.saveSearchHistory();
    }

    /**
     * Highlight search terms in text
     */
    highlightTerms(text, query) {
        if (!query || !text) return text;
        
        const terms = query.toLowerCase().split(/\s+/).filter(term => term.length > 1);
        let highlightedText = text;
        
        terms.forEach(term => {
            const regex = new RegExp(`(${this.escapeRegex(term)})`, 'gi');
            highlightedText = highlightedText.replace(regex, '<mark>$1</mark>');
        });
        
        return highlightedText;
    }

    /**
     * Escape special regex characters
     */
    escapeRegex(string) {
        return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    /**
     * Destroy the search instance
     */
    destroy() {
        this.searchGeneration += 1;
        if (this.pagefind?.destroy) this.pagefind.destroy();
        this.pagefind = null;
        this.isInitialized = false;
    }
}

export { PagefindSearch };
