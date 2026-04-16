use crate::layouts::AppLayout;
use crate::views::{Im, Live, Settings};
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Live {},
    #[route("/im")]
    Im {},
    #[route("/settings")]
    Settings {},
}
