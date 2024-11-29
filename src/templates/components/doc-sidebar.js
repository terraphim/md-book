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
      <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@shoelace-style/shoelace@2.12.0/cdn/themes/light.css" />
      <style>
        :host {
          display: block;
          height: 100%;
        }
        .sidebar-content {
          padding: 1rem;
        }
        .nav-groups {
          margin-top: 1rem;
          list-style: none;
          padding: 0;
        }
        .nav-group {
          margin-bottom: 1.5rem;
        }
        .nav-group-title {
          font-weight: 600;
          color: var(--sl-color-neutral-700);
          margin-bottom: 0.5rem;
        }
        ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }
        a {
          color: var(--sl-color-neutral-700);
          text-decoration: none;
          display: block;
          padding: 0.25rem 0;
        }
        a:hover {
          color: var(--sl-color-primary-600);
        }
        a.active {
          color: var(--sl-color-primary-600);
          font-weight: 500;
        }
      </style>
      <div class="sidebar-content">
        <sl-input placeholder="Search docs..." size="small" clearable>
          <sl-icon name="search" slot="prefix"></sl-icon>
        </sl-input>
        <slot></slot>
      </div>
    `;
  }
}

customElements.define('doc-sidebar', DocSidebar); 