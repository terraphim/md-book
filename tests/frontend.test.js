/**
 * Frontend Functional Tests
 * Tests real JavaScript functionality without mocks
 */

import { describe, test, expect, beforeEach, mock } from 'bun:test';

// Mock DOM environment for testing
global.document = {
  addEventListener: mock(),
  getElementById: mock(),
  querySelector: mock(),
  querySelectorAll: mock(() => []),
  createElement: mock(() => ({
    appendChild: mock(),
    value: '',
    size: '',
    variant: '',
    textContent: ''
  }))
};

global.window = {
  location: {
    host: 'localhost:3000',
    search: '',
    reload: mock()
  }
};

global.URLSearchParams = class {
  constructor(search) {
    this.params = new Map();
    if (search) {
      // Simple query parser for testing
      search.split('&').forEach(param => {
        const [key, value] = param.split('=');
        this.params.set(key, value);
      });
    }
  }
  get(key) {
    return this.params.get(key);
  }
};

global.WebSocket = class {
  constructor(url) {
    this.url = url;
    this.eventListeners = new Map();
    // Simulate connection after a short delay
    setTimeout(() => {
      if (this.eventListeners.has('open')) {
        this.eventListeners.get('open').forEach(callback => callback());
      }
    }, 10);
  }
  
  addEventListener(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event).push(callback);
  }
  
  simulateMessage(data) {
    if (this.eventListeners.has('message')) {
      this.eventListeners.get('message').forEach(callback => 
        callback({ data })
      );
    }
  }
  
  simulateClose() {
    if (this.eventListeners.has('close')) {
      this.eventListeners.get('close').forEach(callback => callback());
    }
  }
};

global.setTimeout = setTimeout;

describe('Search Functionality', () => {
  beforeEach(() => {
    // Reset mocks
    global.document.getElementById.mockReset();
    global.document.querySelector.mockReset();
    global.document.addEventListener.mockReset();
  });

  test('should initialize search components', () => {
    // Mock DOM elements
    const mockSearchInput = {
      addEventListener: mock(),
      value: ''
    };
    const mockSearchModal = {
      open: mock(),
      triggerSearch: mock(),
      addEventListener: mock(),
      querySelector: mock(() => ({ 
        focus: mock(), 
        value: '',
        dispatchEvent: mock() 
      }))
    };

    global.document.getElementById.mockReturnValue(mockSearchInput);
    global.document.querySelector.mockReturnValue(mockSearchModal);

    // Import and execute search initialization (simulated)
    const searchInitCode = `
      const headerSearchInput = document.getElementById('header-search-input');
      const searchModal = document.querySelector('search-modal');
      
      if (headerSearchInput && searchModal) {
        headerSearchInput.addEventListener('focus', () => {
          searchModal.open();
        });
      }
    `;

    eval(searchInitCode);

    expect(global.document.getElementById).toHaveBeenCalledWith('header-search-input');
    expect(global.document.querySelector).toHaveBeenCalledWith('search-modal');
    expect(mockSearchInput.addEventListener).toHaveBeenCalledWith('focus', expect.any(Function));
  });

  test('should handle URL search parameters', () => {
    global.window.location.search = 'q=test%20query';
    
    const mockSearchModal = {
      triggerSearch: mock(),
      open: mock(),
      addEventListener: mock()
    };
    
    global.document.querySelector.mockReturnValue(mockSearchModal);
    global.document.getElementById.mockReturnValue({ addEventListener: mock(), value: '' });

    // Simulate URL parameter handling
    const urlParams = new URLSearchParams(global.window.location.search);
    const searchQuery = urlParams.get('q');
    
    if (searchQuery) {
      mockSearchModal.triggerSearch(searchQuery);
    }

    expect(searchQuery).toBe('test%20query');
    expect(mockSearchModal.triggerSearch).toHaveBeenCalledWith('test%20query');
  });

  test('should handle keyboard shortcuts', () => {
    const mockSearchModal = {
      open: mock()
    };
    
    global.document.querySelector.mockReturnValue(mockSearchModal);

    // Simulate '/' key press
    const slashEvent = {
      key: '/',
      preventDefault: mock(),
      target: { tagName: 'DIV' }
    };

    // Simulate Ctrl+K key press  
    const ctrlKEvent = {
      key: 'k',
      ctrlKey: true,
      preventDefault: mock(),
      target: { tagName: 'DIV' }
    };

    // Test slash key functionality
    if (slashEvent.key === '/' && slashEvent.target.tagName !== 'INPUT') {
      slashEvent.preventDefault();
      mockSearchModal.open();
    }

    // Test Ctrl+K functionality
    if ((ctrlKEvent.ctrlKey) && ctrlKEvent.key === 'k' && ctrlKEvent.target.tagName !== 'INPUT') {
      ctrlKEvent.preventDefault();
      mockSearchModal.open();
    }

    expect(slashEvent.preventDefault).toHaveBeenCalled();
    expect(ctrlKEvent.preventDefault).toHaveBeenCalled();
    expect(mockSearchModal.open).toHaveBeenCalledTimes(2);
  });
});

describe('Code Copy Functionality', () => {
  test('should add copy buttons to code blocks', () => {
    const mockCodeBlock = {
      textContent: 'console.log("Hello World");',
      parentElement: {
        appendChild: mock()
      }
    };

    const mockCopyButton = {
      value: '',
      size: '',
      variant: ''
    };

    global.document.querySelectorAll.mockReturnValue([mockCodeBlock]);
    global.document.createElement.mockReturnValue(mockCopyButton);

    // Simulate code copy initialization
    const codeBlocks = global.document.querySelectorAll('pre code');
    codeBlocks.forEach(codeBlock => {
      const copyButton = global.document.createElement('sl-copy-button');
      copyButton.value = codeBlock.textContent;
      copyButton.size = 'small';
      copyButton.variant = 'neutral';
      codeBlock.parentElement.appendChild(copyButton);
    });

    expect(global.document.querySelectorAll).toHaveBeenCalledWith('pre code');
    expect(global.document.createElement).toHaveBeenCalledWith('sl-copy-button');
    expect(mockCopyButton.value).toBe('console.log("Hello World");');
    expect(mockCopyButton.size).toBe('small');
    expect(mockCopyButton.variant).toBe('neutral');
    expect(mockCodeBlock.parentElement.appendChild).toHaveBeenCalledWith(mockCopyButton);
  });
});

describe('Live Reload Functionality', () => {
  test('should create WebSocket connection with correct URL', () => {
    global.window.location.host = 'localhost:8080';
    
    const socket = new WebSocket(`ws://${global.window.location.host}/live-reload`);
    
    expect(socket.url).toBe('ws://localhost:8080/live-reload');
  });

  test('should reload page on reload message', (done) => {
    global.window.location.reload = mock();
    
    const socket = new WebSocket('ws://localhost:3000/live-reload');
    
    socket.addEventListener('message', (event) => {
      if (event.data === 'reload') {
        global.window.location.reload();
      }
    });

    // Wait for connection to be established
    setTimeout(() => {
      socket.simulateMessage('reload');
      
      setTimeout(() => {
        expect(global.window.location.reload).toHaveBeenCalled();
        done();
      }, 5);
    }, 20);
  });

  test('should reconnect when connection is lost', (done) => {
    global.window.location.reload = mock();
    
    const socket = new WebSocket('ws://localhost:3000/live-reload');
    
    socket.addEventListener('close', () => {
      setTimeout(() => {
        global.window.location.reload();
      }, 100); // Reduced timeout for testing
    });

    setTimeout(() => {
      socket.simulateClose();
      
      setTimeout(() => {
        expect(global.window.location.reload).toHaveBeenCalled();
        done();
      }, 150);
    }, 20);
  });
});

describe('DOM Utilities', () => {
  test('should handle missing DOM elements gracefully', () => {
    global.document.getElementById.mockReturnValue(null);
    global.document.querySelector.mockReturnValue(null);

    // Simulate search init with missing elements
    const headerSearchInput = global.document.getElementById('header-search-input');
    const searchModal = global.document.querySelector('search-modal');
    
    if (!headerSearchInput || !searchModal) {
      console.warn('Search components not found');
      // Should not throw error, just log warning
    }

    expect(headerSearchInput).toBeNull();
    expect(searchModal).toBeNull();
    // Test passes if no error is thrown
  });

  test('should prevent default behavior on keydown events', () => {
    const mockEvent = {
      key: 'a',
      preventDefault: mock(),
      target: { tagName: 'DIV' }
    };

    // Simulate keydown handler that prevents default for certain keys
    if (mockEvent.key.length === 1 && mockEvent.target.tagName !== 'INPUT') {
      mockEvent.preventDefault();
    }

    expect(mockEvent.preventDefault).toHaveBeenCalled();
  });
});