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
      <link rel="stylesheet" href="https://early.webawesome.com/webawesome@3.0.0-beta.6/dist/styles/themes/default.css" />
      <style>
        :host {
          display: block;
          margin: var(--wa-spacing-medium, 1rem) auto;
          max-width: 800px;
        }

        .block-container {
          background: var(--wa-panel-background-color, var(--wa-color-neutral-50));
          border-radius: var(--wa-border-radius-medium, 0.375rem);
          box-shadow: var(--wa-shadow-small, 0 1px 2px 0 rgb(0 0 0 / 0.05));
          padding: var(--wa-spacing-large, 1.5rem);
          text-align: center;
        }

        .header {
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: var(--wa-font-size-small, 0.875rem);
          gap: var(--wa-spacing-medium, 1rem);
          margin-bottom: var(--wa-spacing-medium, 1rem);
        }

        h3 {
          margin: 0;
          font-weight: var(--wa-font-weight-semibold, 500);
          font-size: var(--wa-font-size-small, 0.875rem);
          color: var(--wa-color-neutral-500, #64748b);
        }

        ::slotted(*) {
          text-align: center;
          margin: 0 auto;
        }

        .content {
          color: var(--wa-color-neutral-500, #64748b);
          font-family: var(--wa-font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif);
          line-height: 1.5;
          text-align: center;
        }

        ::slotted(h2) {
          margin: 0 0 var(--wa-spacing-small, 0.5rem) 0;
          font-size: var(--wa-font-size-medium, 1rem);
          font-weight: var(--wa-font-weight-semibold, 500);
          color: var(--wa-color-neutral-500, #64748b);
        }

        ::slotted(h3) {
          margin: 0 0 var(--wa-spacing-small, 0.5rem) 0;
          font-size: var(--wa-font-size-small, 0.875rem);
          font-weight: var(--wa-font-weight-semibold, 500);
          color: var(--wa-color-neutral-500, #64748b);
        }

        ::slotted(wa-button) {
          margin-top: var(--wa-spacing-medium, 1rem);
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
          <wa-icon name="circle-info" style="color: var(--wa-color-neutral-500, #64748b); font-size: var(--wa-font-size-large, 1.125rem);"></wa-icon>
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
