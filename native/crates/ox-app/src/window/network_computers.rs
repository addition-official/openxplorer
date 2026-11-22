// SPDX-License-Identifier: AGPL-3.0-only
//! The computers Network discovered, in the navigation pane (SIDE-019).
//!
//! As in Windows 11's navigation pane (checked in Windows 11 26200),
//! Network lists the computers on the network under it, each with a
//! chevron of its own that lists the computer's shared folders under it.
//! The computers are those discovery found this session
//! ([`NetworkServices::computers`](crate::network::NetworkServices)):
//! going to Network discovers them, and so does expanding Network in the
//! pane, which is the user asking, as in Windows. Nothing is discovered
//! at start. A computer's shared folders are read when its chevron, or
//! Right on its row, expands it; a share that has to be mounted first is
//! mounted, asking for a password through the window's sign-in as
//! opening the computer does.

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use ox_core::entry::{enumerate_folder, EntryError};
use ox_core::location::same_location;
use ox_core::network::DiscoveredServer;

use super::actions::text_action;
use super::sidebar::entries::NetworkComputer;
use super::window_action::WindowAction;
use super::BrowserWindow;

/// The most shared folders listed under one computer.
const MAX_SHARES: usize = 500;

impl BrowserWindow {
    /// Adds `win.toggle-network-computer`.
    pub(super) fn install_network_computers(&self) {
        self.add_action_entries([text_action(WindowAction::ToggleNetworkComputer, |window, uri| {
            // Once the click or key that asked has been handled: the
            // rows are drawn again, which must not happen while the list
            // still handles the event.
            let uri = uri.to_owned();
            glib::idle_add_local_once(glib::clone!(
                #[weak]
                window,
                move || window.toggle_network_computer(&uri)
            ));
        })]);
    }

    /// The computers the navigation pane lists under Network: those
    /// discovery found, but not one Network lists already as a network
    /// location, each with its shared folders while expanded.
    pub(super) fn network_computers(&self, locations: &[String]) -> Vec<NetworkComputer> {
        let expanded = self.imp().expanded_computers.borrow();
        self.context()
            .network()
            .computers()
            .into_iter()
            .filter(|server| !locations.iter().any(|uri| same_location(uri, &server.uri)))
            .map(|server| {
                let shares = expanded
                    .iter()
                    .find(|(uri, _)| *uri == server.uri)
                    .map(|(_, shares)| shares.clone().unwrap_or_default());
                NetworkComputer {
                    uri: server.uri,
                    label: server.label,
                    shares,
                }
            })
            .collect()
    }

    /// Discovery found `servers`: every window lists the new ones.
    pub(super) fn remember_computers(&self, servers: &[DiscoveredServer]) {
        if !self.context().network().remember_computers(servers) {
            return;
        }
        let windows = self.application().map(|app| app.windows()).unwrap_or_default();
        for window in windows {
            if let Ok(window) = window.downcast::<BrowserWindow>() {
                window.render_places();
            }
        }
    }

    /// Network was expanded in the navigation pane: discovers the
    /// computers on the network, as Windows does, unless discovery is
    /// running already.
    pub(super) fn discover_computers_for_pane(&self) {
        if self.imp().discovering_computers.replace(true) {
            return;
        }
        let discoverer = self.context().network().discoverer();
        let window = self.downgrade();
        glib::spawn_future_local(async move {
            let found = crate::network::discover_computers(&discoverer, |servers| {
                if let Some(window) = window.upgrade() {
                    window.remember_computers(&servers);
                }
            })
            .await;
            if let Some(window) = window.upgrade() {
                window.imp().discovering_computers.set(false);
                if let Err(problem) = found {
                    glib::g_debug!(ox_core::LOG_DOMAIN, "Network discovery: {problem}");
                }
            }
        });
    }

    /// Lists the shared folders of the computer at `uri` under it, or
    /// hides them while they are listed.
    fn toggle_network_computer(&self, uri: &str) {
        let was_expanded = {
            let mut expanded = self.imp().expanded_computers.borrow_mut();
            let before = expanded.len();
            expanded.retain(|(shown, _)| shown != uri);
            let was_expanded = expanded.len() != before;
            if !was_expanded {
                expanded.push((uri.to_owned(), None));
            }
            was_expanded
        };
        self.render_places_keeping_focus(uri);
        if was_expanded {
            return;
        }
        let window = self.downgrade();
        let uri = uri.to_owned();
        glib::spawn_future_local(async move {
            let Some(listing) = window.upgrade().map(|window| window.read_shares(&uri)) else {
                return;
            };
            let read = listing.await;
            let Some(window) = window.upgrade() else {
                return;
            };
            window.show_shares(&uri, read);
        });
    }

    /// Shows the shared folders `read` of the computer at `uri`, if it is
    /// still expanded, or why they could not be read.
    fn show_shares(&self, uri: &str, read: Result<Vec<(String, String)>, EntryError>) {
        let message = {
            let mut expanded = self.imp().expanded_computers.borrow_mut();
            let Some(at) = expanded.iter().position(|(shown, _)| shown == uri) else {
                return;
            };
            match read {
                Ok(shares) => {
                    expanded[at].1 = Some(shares);
                    None
                }
                Err(EntryError::Cancelled) => {
                    expanded.remove(at);
                    None
                }
                Err(error) => {
                    expanded.remove(at);
                    Some(error.to_string())
                }
            }
        };
        self.render_places_keeping_focus(uri);
        if let Some(message) = message {
            self.show_message(&message);
        }
    }

    /// Reads the shared folders of the computer at `uri`: its folders,
    /// by name, without hidden ones. A computer that has to be mounted
    /// first is mounted once, asking for a password if it needs one.
    fn read_shares(
        &self,
        uri: &str,
    ) -> impl std::future::Future<Output = Result<Vec<(String, String)>, EntryError>> + 'static {
        let uri = uri.to_owned();
        let window = self.downgrade();
        async move {
            match list_folders(&uri).await {
                Err(error) if error.needs_mount() => {
                    let mounting = window
                        .upgrade()
                        .map(|window| window.network().mount(&uri))
                        .ok_or(EntryError::Cancelled)?;
                    mounting.await.map_err(|error| {
                        if error.is_cancelled() {
                            EntryError::Cancelled
                        } else {
                            EntryError::Failed(error.to_string())
                        }
                    })?;
                    list_folders(&uri).await
                }
                read => read,
            }
        }
    }

    /// Draws the places again, keeping the keyboard on the row of `uri`
    /// when a row of the navigation pane had it.
    fn render_places_keeping_focus(&self, uri: &str) {
        let sidebar = self.sidebar();
        let had_focus = sidebar.list().focus_child().is_some();
        self.render_places();
        if had_focus {
            sidebar.focus_location(uri);
        }
    }
}

/// The folders at `uri`, name and location, by name ignoring case,
/// without hidden ones; at most [`MAX_SHARES`].
async fn list_folders(uri: &str) -> Result<Vec<(String, String)>, EntryError> {
    let mut folders: Vec<(String, String)> = Vec::new();
    enumerate_folder(uri, |entries| {
        let found = entries
            .into_iter()
            .filter(|entry| entry.is_dir && !entry.is_hidden)
            .map(|entry| (entry.name.clone(), entry.navigation_uri().to_owned()));
        folders.extend(found);
    })
    .await?;
    folders.sort_by_key(|(name, _)| name.to_lowercase());
    folders.truncate(MAX_SHARES);
    Ok(folders)
}
