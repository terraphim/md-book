# Lessons Learned

## 2025-12-23: Search and Link Conversion Debugging

### Issue: Pagefind Search Not Working on Deployed Site

**Symptom**: Search modal opens but returns "No results found" for any query.

**Debugging Approach**:
1. Used Playwright to navigate to the deployed site
2. Checked browser console logs - found 404 errors for `/pagefind/pagefind.js`
3. Compared CI workflow (`ci.yml`) with deploy workflow (`deploy.yml`)
4. Found that `npm install -g pagefind` was in CI but not in deploy

**Root Cause**: The `pagefind` CLI must be installed before running the md-book binary because:
- `md-book` calls `PagefindBuilder::build()` which executes `pagefind --site <output_dir>` as a subprocess
- Without the CLI installed, the subprocess silently fails (error is caught with `eprintln!`)
- The build continues but no `/pagefind/` directory is created

**Fix**: Add `npm install -g pagefind` step to deploy workflows before generating the site.

**Lesson**: When a build tool depends on external CLIs, ensure they're installed in ALL workflows that run the tool, not just the CI workflow.

---

### Issue: Internal .md Links Leading to 404s

**Symptom**: Links in SUMMARY.html content area pointed to `.md` files (e.g., `href="page.md"`) instead of `.html`.

**Debugging Approach**:
1. User provided the rendered HTML showing the issue
2. Sidebar navigation was correct (`.html`) but content links were wrong (`.md`)
3. Traced markdown processing in `core.rs` to find where HTML is generated

**Root Cause**: The `markdown` crate converts markdown to HTML but doesn't modify internal link extensions. The SUMMARY.md file contains links like `[Page](page.md)` which get rendered literally.

**Fix**: Added post-processing function `convert_md_links_to_html()` that:
- Finds `href="...md"` patterns in the HTML output
- Skips external links (http://, https://, mailto://, //)
- Converts internal `.md` links to `.html`

**Lesson**: Markdown-to-HTML converters typically don't modify link targets. If you need link transformation, add post-processing.

---

### Issue: Dependabot PR Failing with Breaking Changes

**Symptom**: PR #18 with Rust dependency updates failed CI.

**Root Cause**: `warp` 0.3.5 → 0.4.2 includes breaking API changes to WebSocket module.

**Lesson**: Dependabot groups dependencies together, which can cause PRs to fail when any dependency has breaking changes. Consider:
1. Reviewing changelogs before merging grouped updates
2. Using separate groups for major vs minor/patch updates
3. Adding version constraints in Cargo.toml for dependencies with unstable APIs

---

## Best Practices Discovered

### 1. Silent Error Handling Can Hide Issues
The `PagefindBuilder` used `eprintln!` for errors instead of failing the build:
```rust
if let Err(e) = pagefind.build().await {
    eprintln!("Search indexing failed: {e}");
}
```
This allowed builds to succeed without search functionality. Consider whether errors should be fatal or logged.

### 2. Verify Build Outputs
Added verification step in deploy workflows:
```yaml
if [ -d "dist/pagefind" ]; then
  echo "✓ Pagefind search index created successfully"
else
  echo "✗ ERROR: Pagefind search index not found!"
  exit 1
fi
```
This catches issues before deployment rather than discovering them in production.

### 3. Use Playwright for Debugging Deployed Sites
Playwright's MCP tools are excellent for debugging deployed web apps:
- Navigate to URLs
- Take screenshots
- Check console logs
- Interact with elements
Much faster than manual browser debugging.

### 4. Check Browser Console Logs
The first debugging step for frontend issues should be checking console logs:
```javascript
// Console showed:
Failed to initialize Pagefind: TypeError: Failed to fetch dynamically imported module
```
This immediately pointed to missing files rather than code bugs.

---

## Pitfalls to Avoid

1. **Don't assume CI = Deploy**: Just because CI passes doesn't mean deploy will work - they may have different dependencies installed.

2. **Don't rely on default link handling**: Markdown converters won't automatically adjust links for your build system.

3. **Don't group all dependencies together**: Breaking changes in one dependency can block updates to others.

4. **Don't silence subprocess errors**: If an external tool fails, the build should probably fail too.
