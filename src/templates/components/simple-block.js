// Assets are resolved from this module's own URL, so the component works at a
// domain root, under a sub-path, and over file:// alike. Shadow DOM cannot
// inherit the page's stylesheets, and this file is copied verbatim so it has no
// access to Tera's path_to_root.
const BOOK_ROOT = new URL('../', import.meta.url);
const SHOELACE_THEME = new URL('vendor/shoelace/themes/light.css', BOOK_ROOT).href;
const BOOK_STYLES = new URL('css/styles.css', BOOK_ROOT).href;

class SimpleBlock extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
  }

  render() {
    this.shadowRoot.innerHTML = `
      <link rel="stylesheet" href="${SHOELACE_THEME}" />
      <style>
        :host {
          display: block;
          margin: var(--sl-spacing-medium) auto;
          max-width: 800px;
        }

        .block-container {
          background: var(--sl-panel-background-color);
          border-radius: var(--sl-border-radius-medium);
          box-shadow: var(--sl-shadow-x-small);
          padding: var(--sl-spacing-large);
          text-align: center;
        }

        .header {
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: var(--sl-font-size-small);
          gap: var(--sl-spacing-medium);
          margin-bottom: var(--sl-spacing-medium);
        }
        
        h3 {
          margin: 0;
          font-weight: var(--sl-font-weight-semibold);
          font-size: var(--sl-font-size-small);
          color: var(--sl-color-neutral-500);
        }

        ::slotted(*) {
          text-align: center;
          margin: 0 auto;
        }

        .content {
          color: var(--sl-color-neutral-500);
          font-family: var(--sl-font-sans);
          line-height: var(--sl-line-height-normal);
          text-align: center;
        }

        ::slotted(h2) {
          margin: 0 0 var(--sl-spacing-small) 0;
          font-size: var(--sl-font-size-medium);
          font-weight: var(--sl-font-weight-semibold);
          color: var(--sl-color-neutral-500);
        }

        ::slotted(h3) {
          margin: 0 0 var(--sl-spacing-small) 0;
          font-size: var(--sl-font-size-small);
          font-weight: var(--sl-font-weight-semibold);
          color: var(--sl-color-neutral-500);
        }

        ::slotted(sl-button) {
          margin-top: var(--sl-spacing-medium);
        }

        ::slotted(:last-child) {
          margin-bottom: 0;
        }

        ::slotted(.logo) {
          width: 100%;
          max-width: 300px;
          height: auto;
        }

        @media (max-width: 768px) {
          ::slotted(.logo) {
            max-width: 200px;
          }
        }

        @media (max-width: 480px) {
          ::slotted(.logo) {
            max-width: 150px;
          }
        }
      </style>
      
        <div class="block-container">
          <div class="header">
          <sl-icon name="info-circle" style="color: var(--sl-color-neutral-500); font-size: var(--sl-font-size-large);"></sl-icon>
          <slot name="title"></slot>
        </div>
        <div class="content">
          <center>
            <slot></slot>
          </center>
        </div>
        </div>
    `;
  }
}

customElements.define('simple-block', SimpleBlock); 