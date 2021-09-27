#![recursion_limit="2048"]
use yew::prelude::*;

mod app;
pub mod plugin;


fn main() {
    yew::start_app::<app::App>();
}