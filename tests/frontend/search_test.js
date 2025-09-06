/**
 * Frontend Search Tests
 * Tests for Pagefind search functionality in the browser
 */

// Mock Pagefind for testing
const mockPagefind = {
    init: jest.fn().mockResolvedValue(undefined),
    options: jest.fn().mockResolvedValue(undefined),
    search: jest.fn(),
    preload: jest.fn().mockResolvedValue(undefined),
};

// Mock search results
const mockSearchResults = {
    results: [
        {
            id: '1',
            score: 0.95,
            data: jest.fn().mockResolvedValue({
                url: '/docs/getting-started.html',
                meta: { title: 'Getting Started' },
                excerpt: 'This guide will help you get started quickly...',
                content: 'Full content here',
                sub_results: []
            })
        },
        {
            id: '2', 
            score: 0.87,
            data: jest.fn().mockResolvedValue({
                url: '/docs/advanced.html',
                meta: { title: 'Advanced Topics' },
                excerpt: 'Learn about advanced configuration options...',
                content: 'Advanced content here',
                sub_results: []
            })
        }
    ],
    unfilteredResultCount: 2
};

// Mock DOM environment
document.body.innerHTML = `
    <div id="test-container">
        <search-modal></search-modal>
        <input id="header-search-input" placeholder="Search docs..." />
    </div>
`;

describe('PagefindSearch', () => {
    let PagefindSearch;
    let searchInstance;

    beforeAll(async () => {
        // Mock dynamic import
        global.import = jest.fn().mockResolvedValue(mockPagefind);
        
        // Import our search class
        const module = await import('../../src/templates/js/pagefind-search.js');
        PagefindSearch = module.default || window.PagefindSearch;
    });

    beforeEach(() => {
        searchInstance = new PagefindSearch({
            bundlePath: '/pagefind/',
            debounceDelay: 50, // Faster for tests
            minQueryLength: 2
        });
        
        // Reset mocks
        jest.clearAllMocks();
        mockPagefind.search.mockResolvedValue(mockSearchResults);
        
        // Clear localStorage
        localStorage.clear();
    });

    afterEach(() => {
        if (searchInstance) {
            searchInstance.destroy();
        }
    });

    describe('Initialization', () => {
        test('should initialize with default options', () => {
            const search = new PagefindSearch();
            expect(search.options.bundlePath).toBe('/pagefind/');
            expect(search.options.debounceDelay).toBe(300);
            expect(search.options.minQueryLength).toBe(2);
        });

        test('should initialize with custom options', () => {
            const options = {
                bundlePath: '/custom-pagefind/',
                debounceDelay: 100,
                minQueryLength: 1,
                maxResults: 5
            };
            
            const search = new PagefindSearch(options);
            expect(search.options.bundlePath).toBe('/custom-pagefind/');
            expect(search.options.debounceDelay).toBe(100);
            expect(search.options.minQueryLength).toBe(1);
            expect(search.options.maxResults).toBe(5);
        });

        test('should initialize pagefind library', async () => {
            await searchInstance.init();
            
            expect(global.import).toHaveBeenCalledWith('/pagefind/pagefind.js');
            expect(mockPagefind.options).toHaveBeenCalledWith({
                bundlePath: '/pagefind/',
                baseUrl: '/'
            });
            expect(mockPagefind.init).toHaveBeenCalled();
            expect(searchInstance.isInitialized).toBe(true);
        });
    });

    describe('Search functionality', () => {
        beforeEach(async () => {
            await searchInstance.init();
        });

        test('should perform search and return results', async () => {
            const callback = jest.fn();
            
            await searchInstance.search('getting started', callback);
            
            // Wait for debounce
            await new Promise(resolve => setTimeout(resolve, 60));
            
            expect(mockPagefind.preload).toHaveBeenCalledWith('getting started');
            expect(mockPagefind.search).toHaveBeenCalledWith('getting started');
            expect(callback).toHaveBeenCalledWith({
                query: 'getting started',
                results: expect.arrayContaining([
                    expect.objectContaining({
                        url: '/docs/getting-started.html',
                        title: 'Getting Started',
                        excerpt: 'This guide will help you get started quickly...'
                    })
                ]),
                totalResults: 2,
                unfilteredResultCount: 2
            });
        });

        test('should handle search errors gracefully', async () => {
            mockPagefind.search.mockRejectedValue(new Error('Search failed'));
            
            const callback = jest.fn();
            await searchInstance.search('test query', callback);
            
            // Wait for debounce
            await new Promise(resolve => setTimeout(resolve, 60));
            
            expect(callback).toHaveBeenCalledWith([], expect.any(Error));
        });

        test('should not search for queries below minimum length', async () => {
            const callback = jest.fn();
            
            await searchInstance.search('a', callback);
            
            expect(callback).toHaveBeenCalledWith([]);
            expect(mockPagefind.search).not.toHaveBeenCalled();
        });

        test('should debounce multiple rapid searches', async () => {
            const callback = jest.fn();
            
            // Rapid succession searches
            searchInstance.search('test1', callback);
            searchInstance.search('test2', callback);
            searchInstance.search('test3', callback);
            
            // Wait for debounce
            await new Promise(resolve => setTimeout(resolve, 60));
            
            // Should only search for the last query
            expect(mockPagefind.search).toHaveBeenCalledTimes(1);
            expect(mockPagefind.search).toHaveBeenCalledWith('test3');
        });
    });

    describe('URL handling', () => {
        test('should extract query from URL parameters', () => {
            // Mock URL with search parameter
            delete window.location;
            window.location = { search: '?q=test%20query&other=value' };
            
            const query = searchInstance.handleUrlParams();
            expect(query).toBe('test query');
        });

        test('should return null for URLs without query parameter', () => {
            delete window.location;
            window.location = { search: '?other=value' };
            
            const query = searchInstance.handleUrlParams();
            expect(query).toBeNull();
        });

        test('should update URL with search query', () => {
            // Mock URL and history
            delete window.location;
            window.location = new URL('https://example.com/');
            window.history = { replaceState: jest.fn() };
            
            searchInstance.updateUrl('test query');
            
            expect(window.history.replaceState).toHaveBeenCalledWith(
                {},
                '',
                expect.stringContaining('q=test%20query')
            );
        });
    });

    describe('Search history', () => {
        test('should save and load search history', () => {
            searchInstance.addToSearchHistory('first query');
            searchInstance.addToSearchHistory('second query');
            
            const history = searchInstance.getSearchHistory();
            expect(history).toEqual(['second query', 'first query']);
        });

        test('should persist search history in localStorage', () => {
            searchInstance.addToSearchHistory('persistent query');
            
            // Create new instance to test persistence
            const newInstance = new PagefindSearch();
            const history = newInstance.getSearchHistory();
            
            expect(history).toContain('persistent query');
        });

        test('should limit search history size', () => {
            // Add more than 10 items
            for (let i = 0; i < 15; i++) {
                searchInstance.addToSearchHistory(`query ${i}`);
            }
            
            const history = searchInstance.getSearchHistory();
            expect(history.length).toBe(10);
            expect(history[0]).toBe('query 14'); // Most recent
        });

        test('should clear search history', () => {
            searchInstance.addToSearchHistory('query to clear');
            searchInstance.clearSearchHistory();
            
            const history = searchInstance.getSearchHistory();
            expect(history).toEqual([]);
        });
    });

    describe('Text highlighting', () => {
        test('should highlight search terms in text', () => {
            const text = 'This is a test document with some content';
            const query = 'test content';
            
            const highlighted = searchInstance.highlightTerms(text, query);
            
            expect(highlighted).toContain('<mark>test</mark>');
            expect(highlighted).toContain('<mark>content</mark>');
        });

        test('should handle empty query gracefully', () => {
            const text = 'Some text';
            const highlighted = searchInstance.highlightTerms(text, '');
            
            expect(highlighted).toBe(text);
        });

        test('should escape regex special characters', () => {
            const escaped = searchInstance.escapeRegex('test.*+?^${}()|[]\\');
            expect(escaped).toBe('test\\.\\*\\+\\?\\^\\$\\{\\}\\(\\)\\|\\[\\]\\\\');
        });
    });
});

describe('SearchModal', () => {
    let searchModal;

    beforeEach(() => {
        // Create search modal element
        document.body.innerHTML = `
            <search-modal></search-modal>
            <input id="header-search-input" />
        `;
        
        searchModal = document.querySelector('search-modal');
        
        // Mock PagefindSearch
        window.PagefindSearch = class MockPagefindSearch {
            constructor(options) {
                this.options = options;
                this.isInitialized = false;
            }
            
            async init() {
                this.isInitialized = true;
            }
            
            async search(query, callback) {
                setTimeout(() => {
                    callback({
                        query,
                        results: mockSearchResults.results.map(r => ({
                            url: '/test.html',
                            title: 'Test Result',
                            excerpt: 'Test excerpt',
                            score: 0.9
                        })),
                        totalResults: 1
                    });
                }, 10);
            }
            
            handleUrlParams() {
                return null;
            }
            
            updateUrl(query) {}
            
            highlightTerms(text, query) {
                return text.replace(new RegExp(`(${query})`, 'gi'), '<mark>$1</mark>');
            }
            
            destroy() {}
        };
    });

    afterEach(() => {
        document.body.innerHTML = '';
    });

    test('should initialize and render modal structure', async () => {
        // Trigger connected callback
        searchModal.connectedCallback();
        
        expect(searchModal.querySelector('.search-modal-overlay')).toBeTruthy();
        expect(searchModal.querySelector('.search-input')).toBeTruthy();
        expect(searchModal.querySelector('.search-results')).toBeTruthy();
        expect(searchModal.querySelector('.search-close-btn')).toBeTruthy();
    });

    test('should open and close modal', async () => {
        searchModal.connectedCallback();
        
        // Test opening
        searchModal.open();
        expect(searchModal.isOpen).toBe(true);
        expect(searchModal.overlay.style.display).toBe('flex');
        
        // Test closing
        searchModal.close();
        expect(searchModal.isOpen).toBe(false);
        expect(searchModal.overlay.style.display).toBe('none');
    });

    test('should handle keyboard shortcuts', async () => {
        searchModal.connectedCallback();
        
        // Test '/' key
        const slashEvent = new KeyboardEvent('keydown', { key: '/' });
        document.dispatchEvent(slashEvent);
        
        expect(searchModal.isOpen).toBe(true);
        
        // Test Escape key
        const escapeEvent = new KeyboardEvent('keydown', { key: 'Escape' });
        document.dispatchEvent(escapeEvent);
        
        expect(searchModal.isOpen).toBe(false);
    });

    test('should perform search and display results', async () => {
        searchModal.connectedCallback();
        await searchModal.initializeSearch();
        
        searchModal.performSearch('test query');
        
        // Wait for async search
        await new Promise(resolve => setTimeout(resolve, 50));
        
        const results = searchModal.querySelectorAll('.search-result-item');
        expect(results.length).toBeGreaterThan(0);
    });

    test('should handle empty search results', async () => {
        searchModal.connectedCallback();
        
        // Mock empty results
        window.PagefindSearch = class MockPagefindSearch {
            async search(query, callback) {
                callback({ query, results: [], totalResults: 0 });
            }
            handleUrlParams() { return null; }
            updateUrl() {}
            destroy() {}
        };
        
        await searchModal.initializeSearch();
        searchModal.performSearch('nonexistent');
        
        await new Promise(resolve => setTimeout(resolve, 10));
        
        expect(searchModal.emptyElement.style.display).toBe('flex');
    });
});

describe('Search Integration', () => {
    test('should connect header input with search modal', () => {
        document.body.innerHTML = `
            <input id="header-search-input" placeholder="Search..." />
            <search-modal></search-modal>
        `;
        
        const headerInput = document.getElementById('header-search-input');
        const searchModal = document.querySelector('search-modal');
        
        // Mock modal methods
        searchModal.open = jest.fn();
        searchModal.triggerSearch = jest.fn();
        
        // Simulate search initialization script
        headerInput.addEventListener('focus', () => searchModal.open());
        headerInput.addEventListener('input', (e) => {
            if (e.target.value.trim()) {
                searchModal.triggerSearch(e.target.value);
            }
        });
        
        // Test focus opens modal
        headerInput.dispatchEvent(new Event('focus'));
        expect(searchModal.open).toHaveBeenCalled();
        
        // Test input triggers search
        headerInput.value = 'test query';
        headerInput.dispatchEvent(new Event('input'));
        expect(searchModal.triggerSearch).toHaveBeenCalledWith('test query');
    });
});