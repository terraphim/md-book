class DocToc extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this.generateToc();
  }

  generateToc() {
    const article = document.querySelector('.main-article');
    if (!article) return;

    const headers = Array.from(article.querySelectorAll('h1, h2, h3, h4, h5, h6'));
    const tocList = document.createElement('ul');
    tocList.className = 'toc-list';

    headers.forEach(header => {
      // Skip the main title
      if (header.tagName === 'H1' && header === article.querySelector('h1')) {
        return;
      }

      const level = parseInt(header.tagName.charAt(1));
      const title = header.textContent;
      const id = this.slugify(title);

      // Add id to the header if it doesn't have one
      if (!header.id) {
        header.id = id;
      }

      const listItem = document.createElement('li');
      listItem.className = `toc-item level-${level}`;

      const link = document.createElement('a');
      link.href = `#${id}`;
      link.textContent = title;

      listItem.appendChild(link);
      tocList.appendChild(listItem);
    });

    const tocContent = this.shadowRoot.querySelector('.toc-content');
    tocContent.innerHTML = '';
    tocContent.appendChild(tocList);
  }

  slugify(text) {
    return text.toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/(^-|-$)/g, '');
  }

  render() {
    this.shadowRoot.innerHTML = `
      <link rel="stylesheet" href="https://early.webawesome.com/webawesome@3.0.0-beta.6/dist/styles/themes/default.css" />
      <link rel="stylesheet" href="/css/styles.css">
      <style>
        :host {
          display: block;
          padding: var(--wa-spacing-medium, 1rem);
          background: var(--wa-panel-background-color, var(--wa-color-neutral-50));
          border-left: solid 1px var(--wa-panel-border-color, var(--wa-color-neutral-200));
        }

        .toc-header {
          font-family: var(--wa-font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif);
          font-size: var(--wa-font-size-small, 0.875rem);
          font-weight: var(--wa-font-weight-semibold, 500);
          text-transform: uppercase;
          color: var(--wa-color-neutral-500, #64748b);
          margin-bottom: var(--wa-spacing-medium, 1rem);
          letter-spacing: 0.05em;
        }

        .toc-list {
          list-style: none;
          padding: 0;
          margin: 0;
          font-family: var(--wa-font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif);
        }

        .toc-item {
          margin: var(--wa-spacing-small, 0.5rem) 0;
        }

        .toc-item.level-1 { padding-left: 0; }
        .toc-item.level-2 { padding-left: var(--wa-spacing-large, 1.5rem); }
        .toc-item.level-3 { padding-left: calc(var(--wa-spacing-large, 1.5rem) * 2); }
        .toc-item.level-4 { padding-left: calc(var(--wa-spacing-large, 1.5rem) * 3); }
        .toc-item.level-5 { padding-left: calc(var(--wa-spacing-large, 1.5rem) * 4); }
        .toc-item.level-6 { padding-left: calc(var(--wa-spacing-large, 1.5rem) * 5); }

        a {
          color: var(--wa-color-neutral-700, #334155);
          text-decoration: none;
          font-size: var(--wa-font-size-small, 0.875rem);
          line-height: 1.5;
          transition: color 0.15s ease;
        }

        a:hover {
          color: var(--wa-color-primary-600, #0284c7);
        }

        @media (max-width: 1200px) {
          :host {
            display: none;
          }
        }
      </style>
      <div class="toc-header">On this page</div>
      <div class="toc-content"></div>
    `;
  }
}

customElements.define('doc-toc', DocToc);
