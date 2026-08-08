(function () {
  const KEY = 'md-book-theme';
  const root = document.documentElement;
  const stored = localStorage.getItem(KEY);
  const preferredDark = root.dataset.preferredDarkTheme || 'navy';
  const defaultTheme = root.dataset.defaultTheme || 'light';
  function apply(theme) {
    root.setAttribute('data-theme', theme);
    localStorage.setItem(KEY, theme);
  }
  if (stored) {
    apply(stored);
  } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    apply(preferredDark);
  } else {
    apply(defaultTheme);
  }
  document.addEventListener('click', function (e) {
    const btn = e.target.closest('[data-theme-set]');
    if (btn) apply(btn.getAttribute('data-theme-set'));
  });
})();
