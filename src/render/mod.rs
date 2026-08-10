//! Rendering stages: markdown → HTML fragment, then HTML page assembly.

pub mod html;
pub mod markdown;
pub mod slug;

pub use html::{
    copy_static_assets, escape_url_attr, extract_title, init_tera, logo_url, path_to_root,
    render_index, render_page, to_url_path, Section,
};
pub use markdown::{
    convert_md_links_to_html, has_mermaid_fence, render_markdown, RenderedMarkdown,
};
pub use slug::{inject_heading_ids, inject_heading_ids_with, slugify};
