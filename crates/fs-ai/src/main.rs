#![deny(clippy::all, clippy::pedantic)]
#![deny(warnings)]

fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_ai::AiManagerApp);
}
