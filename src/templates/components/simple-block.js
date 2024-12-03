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
      <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@shoelace-style/shoelace@2.12.0/cdn/themes/light.css" />
      <link rel="stylesheet" href="/css/styles.css">
      <style>
        :host {
          display: block;
          margin: var(--sl-spacing-medium) 0;
        }

        .block-container {
          background: var(--sl-panel-background-color);
          border-radius: var(--sl-border-radius-medium);
          box-shadow: var(--sl-shadow-x-small);
          padding: var(--sl-spacing-large);
        }

        .header {
          display: flex;
          align-items: center;
          gap: var(--sl-spacing-medium);
          margin-bottom: var(--sl-spacing-medium);
        }

        .icon {
          color: var(--sl-color-primary-600);
          font-size: var(--sl-font-size-large);
        }

        ::slotted(h1),
        ::slotted(h2),
        ::slotted(h3) {
          margin: 0;
          font-family: var(--sl-font-sans);
          color: var(--sl-color-neutral-900);
        }

        .content {
          color: var(--sl-color-neutral-700);
          font-family: var(--sl-font-sans);
          line-height: var(--sl-line-height-normal);
        }
      </style>

      <div class="block-container">
        <div class="header">
          <div class="icon">ℹ️</div>
          <slot name="title"></slot>
        </div>
        <div class="content">
          <slot></slot>
        </div>
      </div>
    `;
  }
}

customElements.define('simple-block', SimpleBlock); 