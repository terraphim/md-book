// Local Shoelace loader.
//
// Replaces the CDN autoloader so generated books work offline and under any
// sub-path. Only the components md-book actually uses are imported, and only
// the icons it references are vendored (see assets/icons/), which keeps this
// tree at a few hundred KB rather than the 14MB full distribution.
//
// The base path is derived from this module's own URL, so it resolves correctly
// whether the book is served from a domain root, a sub-path, or file://.
import { setBasePath } from './utilities/base-path.js';

setBasePath(new URL('.', import.meta.url).href);

import './components/button/button.js';
import './components/icon/icon.js';
import './components/input/input.js';
import './components/spinner/spinner.js';
import './components/card/card.js';
