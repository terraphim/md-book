class DocToc extends HTMLElement {
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
        }
        .toc-header {
          font-size: 0.875rem;
          font-weight: 600;
          text-transform: uppercase;
          color: var(--sl-color-neutral-600);
          margin-bottom: 0.5rem;
        }
        ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }
        li {
          margin: 0.25rem 0;
        }
        a {
          color: var(--sl-color-neutral-700);
          text-decoration: none;
          font-size: 0.875rem;
          display: block;
          padding: 0.25rem 0;
        }
        a:hover {
          color: var(--sl-color-primary-600);
        }
        ul ul {
          padding-left: 1rem;
        }
      </style>
      <div class="toc-header">On this page</div>
      <slot></slot>
    `;
  }
}

customElements.define('doc-toc', DocToc); 