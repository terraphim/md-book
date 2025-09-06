/**
 * Test setup for frontend tests
 */

// Mock localStorage
const localStorageMock = {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
    clear: jest.fn(),
};
global.localStorage = localStorageMock;

// Mock Web APIs that might not be available in jsdom
global.ResizeObserver = jest.fn().mockImplementation(() => ({
    observe: jest.fn(),
    unobserve: jest.fn(),
    disconnect: jest.fn(),
}));

// Mock requestAnimationFrame
global.requestAnimationFrame = jest.fn(cb => setTimeout(cb, 0));
global.cancelAnimationFrame = jest.fn();

// Mock URL constructor
global.URL = URL;

// Mock console methods to reduce noise in tests
global.console = {
    ...console,
    warn: jest.fn(),
    error: jest.fn(),
    log: jest.fn(),
};

// Mock custom elements
global.customElements = {
    define: jest.fn(),
    get: jest.fn(),
    upgrade: jest.fn(),
    whenDefined: jest.fn(() => Promise.resolve()),
};

// Mock HTMLElement for custom elements
global.HTMLElement = class MockHTMLElement {
    constructor() {
        this.children = [];
        this.innerHTML = '';
        this.textContent = '';
        this.style = {};
        this.classList = {
            add: jest.fn(),
            remove: jest.fn(),
            contains: jest.fn(),
            toggle: jest.fn(),
        };
    }

    querySelector(selector) {
        return null;
    }

    querySelectorAll(selector) {
        return [];
    }

    appendChild(child) {
        this.children.push(child);
    }

    removeChild(child) {
        const index = this.children.indexOf(child);
        if (index > -1) {
            this.children.splice(index, 1);
        }
    }

    addEventListener(event, handler) {
        // Mock implementation
    }

    removeEventListener(event, handler) {
        // Mock implementation
    }

    setAttribute(name, value) {
        this[name] = value;
    }

    getAttribute(name) {
        return this[name];
    }

    focus() {
        // Mock focus
    }

    scrollIntoView() {
        // Mock scroll
    }
};

// Setup DOM environment
beforeEach(() => {
    document.head.innerHTML = '';
    document.body.innerHTML = '';
    
    // Reset localStorage mock
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    localStorageMock.removeItem.mockClear();
    localStorageMock.clear.mockClear();
});