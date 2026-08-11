// Sidebar folding. The server decides the initial state ([output.html.fold],
// with the branch containing the current page left open); this only handles
// the reader toggling it afterwards.
(function () {
  document.addEventListener('click', function (e) {
    const toggle = e.target.closest('.fold-toggle');
    if (!toggle) return;

    const item = toggle.closest('.sidebar-item');
    if (!item) return;

    // The sub-list is the next sibling <ul> after this item.
    const list = item.querySelector(':scope > ul.sidebar-items') || item.nextElementSibling;
    if (!list || list.tagName !== 'UL') return;

    const nowFolded = !list.classList.contains('folded');
    list.classList.toggle('folded', nowFolded);
    toggle.setAttribute('aria-expanded', nowFolded ? 'false' : 'true');
  });
})();
