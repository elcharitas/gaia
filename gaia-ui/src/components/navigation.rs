//! Navigation component for the Gaia UI

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::bs_icons::{BsDiagram3Fill, BsGraphUp, BsHouseFill, BsListTask};
use dioxus_router::prelude::*;

use crate::Route;

/// Main navigation component for the application
#[component]
pub fn Navigation() -> Element {
    let route = use_route::<Route>();

    rsx! {
        nav { class: "bg-gradient-to-r from-blue-600 via-purple-600 to-pink-500 shadow-lg sticky top-0 z-50",
            div { class: "container mx-auto px-4",
                div { class: "flex items-center justify-between h-16",
                    // Logo/Brand
                    div { class: "flex items-center space-x-3",
                        Icon { width: 28, height: 28, icon: BsDiagram3Fill, class: "text-white" }
                        Link { to: Route::Home {}, class: "text-2xl font-extrabold text-white tracking-wide", "Gaia" }
                    }

                    // Navigation Links
                    div { class: "hidden md:block",
                        div { class: "ml-10 flex items-center space-x-4",
                            NavLink {
                                to: Route::Home {},
                                active: matches!(route, Route::Home {..}),
                                icon: rsx! { Icon { width: 20, height: 20, icon: BsHouseFill, class: "" } },
                                label: "Dashboard"
                            }
                            NavLink {
                                to: Route::Pipelines {},
                                active: matches!(route, Route::Pipelines {..}) || matches!(route, Route::PipelineDetails {..}),
                                icon: rsx! { Icon { width: 20, height: 20, icon: BsDiagram3Fill, class: "" } },
                                label: "Pipelines"
                            }
                            NavLink {
                                to: Route::Tasks {},
                                active: matches!(route, Route::Tasks {..}) || matches!(route, Route::TaskDetails {..}),
                                icon: rsx! { Icon { width: 20, height: 20, icon: BsListTask, class: "" } },
                                label: "Tasks"
                            }
                            NavLink {
                                to: Route::Metrics {},
                                active: matches!(route, Route::Metrics {..}),
                                icon: rsx! { Icon { width: 20, height: 20, icon: BsGraphUp, class: "" } },
                                label: "Metrics"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual navigation link component
#[component]
fn NavLink(to: Route, active: bool, icon: Element, label: &'static str) -> Element {
    let active_class = if active {
        "bg-blue-100 text-blue-700"
    } else {
        "text-gray-600 hover:bg-gray-100 hover:text-gray-900"
    };

    rsx! {
        Link {
            to: to,
            class: "px-3 py-2 rounded-md text-sm font-medium flex items-center space-x-2 {active_class}",
            div { class: "flex items-center",
                span { class: "mr-2", {icon} }
                span { "{label}" }
            }
        }
    }
}
