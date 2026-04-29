//! API to interact with tmux sessions.

mod inspect;
mod manage;
mod restore;

pub use inspect::{capture_preview, get_session, get_session_name, list_sessions};
pub use manage::{
    attach_to_session, attach_to_window, close_session, close_window, create_session, create_session_with_path,
    create_window, create_window_with_path, rename_session, rename_window, session_exists,
};
pub use restore::restore_session;
