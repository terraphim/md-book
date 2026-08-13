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
      <style>
        :host {
          display: block;
          margin: var(--wa-spacing-medium) auto;
          max-width: 800px;
        }

        .block-container {
          background: var(--wa-panel-background-color);
          border-radius: var(--wa-border-radius-medium);
          box-shadow: var(--wa-shadow-x-small);
          padding: var(--wa-spacing-large);
          text-align: center;
        }

        .header {
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: var(--wa-font-size-small);
          gap: var(--wa-spacing-medium);
          margin-bottom: var(--wa-spacing-medium);
        }

        h3 {
          margin: 0;
          font-weight: var(--wa-font-weight-semibold);
          font-size: var(--wa-font-size-small);
          color: var(--wa-color-neutral-500);
        }

        ::slotted(*) {
          text-align: center;
          margin: 0 auto;
        }

        .content {
          color: var(--wa-color-neutral-500);
          font-family: var(--wa-font-sans);
          line-height: var(--wa-line-height-normal);
          text-align: center;
        }

        ::slotted(h2) {
          margin: 0 0 var(--wa-spacing-small) 0;
          font-size: var(--wa-font-size-medium);
          font-weight: var(--wa-font-weight-semibold);
          color: var(--wa-color-neutral-500);
        }

        ::slotted(h3) {
          margin: 0 0 var(--wa-spacing-small) 0;
          font-size: var(--wa-font-size-small);
          font-weight: var(--wa-font-weight-semibold);
          color: var(--wa-color-neutral-500);
        }

        ::slotted(wa-button) {
          margin-top: var(--wa-spacing-medium);
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
          <wa-icon name="info-circle" style="color: var(--wa-color-neutral-500); font-size: var(--wa-font-size-large);"></wa-icon>
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
