(function () {
  document.addEventListener('keydown', function (e) {
    if (e.target.matches('input, textarea, [contenteditable]')) return;
    const prev = document.querySelector('.nav-previous');
    const next = document.querySelector('.nav-next');
    if (e.key === 'ArrowLeft' && prev) { window.location = prev.getAttribute('href'); }
    if (e.key === 'ArrowRight' && next) { window.location = next.getAttribute('href'); }
    if (e.key === 's' || e.key === '/') {
      e.preventDefault();
      const modal = document.querySelector('search-modal');
      if (modal && modal.show) modal.show();
      else {
        const input = document.querySelector('input[type="search"], #search');
        if (input) input.focus();
      }
    }
    if (e.key === '?') {
      alert('Keyboard shortcuts:\n← / →  previous / next page\ns or /  search\n?  this help');
    }
  });
})();
