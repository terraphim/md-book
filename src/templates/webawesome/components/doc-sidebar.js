class DocSidebar extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
  }

  render() {
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          padding: var(--wa-spacing-medium);
          background: var(--wa-panel-background-color);
          border-right: solid var(--wa-panel-border-width) var(--wa-panel-border-color);
        }

        /* Reset all list styles */
        ::slotted(ul),
        ::slotted(li) {
          list-style: none;
          list-style-type: none;
          padding: 0;
          margin: 0;
        }

        .sidebar-nav {
          padding: var(--wa-spacing-medium);
        }

        .sidebar-section {
          margin-bottom: var(--wa-spacing-large);
        }

        .sidebar-section-title {
          font-family: var(--wa-font-sans);
          font-size: var(--wa-font-size-small);
          font-weight: var(--wa-font-weight-semibold);
          text-transform: uppercase;
          color: var(--wa-color-neutral-500);
          margin-bottom: var(--wa-spacing-medium);
        }

        .sidebar-items {
          margin: 0 0 0 var(--wa-spacing-medium);
        }

        .sidebar-item {
          margin: var(--wa-spacing-x-small) 0;
        }

        /* Reset all link styles */
        ::slotted(a),
        ::slotted(a:visited) {
          color: var(--wa-color-neutral-700) !important;
          text-decoration: none !important;
        }

        .sidebar-item a {
          display: flex;
          align-items: center;
          gap: var(--wa-spacing-medium);
          color: var(--wa-color-neutral-700);
          text-decoration: none;
          font-size: var(--wa-font-size-medium);
          line-height: var(--wa-line-height-normal);
          padding: var(--wa-spacing-x-small) var(--wa-spacing-medium);
          border-radius: var(--wa-border-radius-medium);
          transition: var(--wa-transition-medium) background-color,
                    var(--wa-transition-medium) color;
          position: relative;
        }

        .sidebar-item wa-icon {
          font-size: 1em;
          color: var(--wa-color-neutral-400);
          transition: var(--wa-transition-medium) color;
        }

        /* Hover state */
        .sidebar-item a:hover {
          background: var(--wa-color-neutral-100);
          color: var(--wa-color-primary-600);
          text-decoration: none;
        }

        .sidebar-item a:hover wa-icon {
          color: var(--wa-color-primary-600);
        }

        /* Active state */
        .sidebar-item a.active {
          background: var(--wa-color-primary-100);
          color: var(--wa-color-primary-600);
          font-weight: var(--wa-font-weight-semibold);
          text-decoration: none;
        }

        .sidebar-item a.active wa-icon {
          color: var(--wa-color-primary-600);
        }

        .sidebar-item a.active::before {
          content: '';
          position: absolute;
          left: calc(-1 * var(--wa-spacing-x-small));
          top: 0;
          bottom: 0;
          width: 3px;
          background: var(--wa-color-primary-600);
          border-radius: 0 var(--wa-border-radius-medium) var(--wa-border-radius-medium) 0;
        }

        @media (max-width: 768px) {
          :host {
            display: none;
          }
        }
      </style>
      <nav class="sidebar-nav">
        <slot></slot>
      </nav>
    `;

    // Only keep page icons code
    this.shadowRoot.host.querySelectorAll('.sidebar-item a').forEach(link => {
      const icon = document.createElement('wa-icon');
      if (link.matches('.active')) {
        icon.name = 'bookmark-fill';
      } else {
        icon.name = 'chevron-right';
      }
      link.prepend(icon);
    });
  }
}

customElements.define('doc-sidebar', DocSidebar);
