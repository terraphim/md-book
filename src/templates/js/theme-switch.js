(function () {
  const KEY = 'md-book-theme';
  const root = document.documentElement;
  const stored = localStorage.getItem(KEY);
  const themes = new Set(['light', 'rust', 'coal', 'navy', 'ayu']);
  const preferredDark = root.dataset.preferredDarkTheme || 'navy';
  const defaultTheme = root.dataset.defaultTheme || 'light';
  function apply(theme) {
    if (!themes.has(theme)) theme = themes.has(defaultTheme) ? defaultTheme : 'light';
    root.setAttribute('data-theme', theme);
    localStorage.setItem(KEY, theme);
    // Reflect the choice for assistive tech, and close the picker.
    document.querySelectorAll('[data-theme-set]').forEach(function (b) {
      if (b.getAttribute('data-theme-set') === theme) {
        b.setAttribute('aria-current', 'true');
      } else {
        b.removeAttribute('aria-current');
      }
    });
  }
  if (themes.has(stored)) {
    apply(stored);
  } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    apply(preferredDark);
  } else {
    apply(defaultTheme);
  }
  document.addEventListener('click', function (e) {
    const btn = e.target.closest('[data-theme-set]');
    if (!btn) return;
    apply(btn.getAttribute('data-theme-set'));
    const menu = btn.closest('details');
    if (menu) menu.open = false;
  });

  // The buttons may not exist yet when this runs (script order, or a custom
  // template); mark the active one once the DOM is ready.
  document.addEventListener('DOMContentLoaded', function () {
    apply(localStorage.getItem(KEY) || root.getAttribute('data-theme') || defaultTheme);
  });
})();
