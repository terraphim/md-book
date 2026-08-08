//! Rendering stages: markdown → HTML fragment, then HTML page assembly.

pub mod html;
pub mod markdown;

pub use html::{copy_static_assets, extract_title, init_tera, render_index, render_page, Section};
pub use markdown::{convert_md_links_to_html, render_markdown};
