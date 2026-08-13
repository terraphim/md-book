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
      <style>
        :host {
          display: block;
          padding: var(--wa-spacing-medium);
          background: var(--wa-panel-background-color);
          border-left: solid var(--wa-panel-border-width) var(--wa-panel-border-color);
        }

        .toc-header {
          font-family: var(--wa-font-sans);
          font-size: var(--wa-font-size-small);
          font-weight: var(--wa-font-weight-semibold);
          text-transform: uppercase;
          color: var(--wa-color-neutral-500);
          margin-bottom: var(--wa-spacing-medium);
        }

        .toc-list {
          list-style: none;
          padding: 0;
          margin: 0;
          font-family: var(--wa-font-sans);
        }

        .toc-item {
          margin: var(--wa-spacing-2x-small) 0;
        }

        .toc-item.level-1 { padding-left: 0; }
        .toc-item.level-2 { padding-left: var(--wa-spacing-large); }
        .toc-item.level-3 { padding-left: calc(var(--wa-spacing-large) * 2); }
        .toc-item.level-4 { padding-left: calc(var(--wa-spacing-large) * 3); }
        .toc-item.level-5 { padding-left: calc(var(--wa-spacing-large) * 4); }
        .toc-item.level-6 { padding-left: calc(var(--wa-spacing-large) * 5); }

        a {
          color: var(--wa-color-neutral-700);
          text-decoration: none;
          font-size: var(--wa-font-size-small);
          line-height: var(--wa-line-height-normal);
          transition: var(--wa-transition-fast) color;
        }

        a:hover {
          color: var(--wa-color-primary-600);
        }

        @media (max-width: 1200px) {
          :host {
            display: none;
          }
        }
      </style>
      <nav class="toc-nav" aria-labelledby="toc-header">
        <h2 class="toc-header" id="toc-header">On this page</h2>
        <div class="toc-content"></div>
      </nav>
    `;
  }
}

customElements.define('doc-toc', DocToc);
