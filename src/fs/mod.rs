use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(not(feature = "wasm"))] {
mod serve_dir;
mod serve_file;

pub(crate) use serve_dir::ServeDir;
pub(crate) use serve_file::ServeFile;
    }
}

