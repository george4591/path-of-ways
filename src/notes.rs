mod context;
mod editor;
mod highlight;
mod lightbox;
mod markdown;
mod modals;
mod model;
mod sidebar;
mod storage;
mod templates;
mod view;

pub use markdown::{render_inline_md, render_markdown};
pub use model::Note;
pub use storage::{load_notes, save_one};
pub use templates::Template;
pub use view::Notes;
