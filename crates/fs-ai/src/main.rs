#![deny(clippy::all, clippy::pedantic, warnings)]

fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_ai::AiManagerApp);
}
