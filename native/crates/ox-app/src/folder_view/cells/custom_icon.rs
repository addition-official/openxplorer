// SPDX-License-Identifier: AGPL-3.0-only
//! Custom icons (PROP-016): an item whose `GVfs` metadata names a picture
//! (`metadata::custom-icon`, which Files reads and writes too) shows that
//! picture in place of its art in the details and icon views. The lookup
//! runs when a cell is bound, at low priority, like a thumbnail lookup,
//! and a picture that arrives after the cell shows another item is
//! dropped. Pictures are decoded once at icon size and kept, so scrolling
//! back to an item does not decode its picture again. An item without a
//! custom icon shows its preview in the same place, when it has one
//! ([`crate::thumbnails`]). A folder without one shows the picture chosen
//! for it ([`super::folder_picture`]). Last comes a picture the user chose
//! in File icons for the item's kind (LOOK-029).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};

use super::folder_picture::{FOLDER_PICTURE, SMALLEST_PICTURE_ICON};
use super::FileCell;
use crate::folder_view::item::FileItem;
use crate::picture_guard::{self, Limits};
use crate::thumbnails::{self, PreviewPolicy};

/// The metadata key Files stores a custom icon under.
pub(crate) const CUSTOM_ICON: &str = "metadata::custom-icon";
/// Pictures larger than this are not decoded for an icon.
const MAX_ICON_BYTES: u64 = 16 * 1024 * 1024;
/// The size a picture is decoded at: the largest icon the views draw.
const ICON_PIXELS: i32 = 256;
/// How many decoded pictures are kept.
const MAX_CACHED_ICONS: usize = 256;
/// How long a tile whose thumbnail the cache holds waits for it without
/// its type icon: a read of the cache takes a few milliseconds, a busy
/// disk longer.
const ICON_WAIT: std::time::Duration = std::time::Duration::from_millis(250);
/// How many custom pictures are decoded at once, including workers whose
/// rows scrolled away before their blocking decoder could finish.
const DECODING_AT_ONCE: usize = 4;

thread_local! {
    /// Decoded pictures by their URI.
    static DECODED: RefCell<HashMap<String, gdk::Texture>> = RefCell::new(HashMap::new());
    static DECODING: Rc<thumbnails::Slots> = Rc::new(thumbnails::Slots::new(DECODING_AT_ONCE));
}

/// The decoded picture at `icon_uri`, decoding it off the main thread the
/// first time.
pub(crate) async fn icon_texture(icon_uri: String) -> Option<gdk::Texture> {
    if let Some(texture) = DECODED.with_borrow(|decoded| decoded.get(&icon_uri).cloned()) {
        return Some(texture);
    }
    let uri = icon_uri.clone();
    let texture = DECODING
        .with(Rc::clone)
        .blocking(move |_| decode(&uri))
        .await
        .flatten()?;
    DECODED.with_borrow_mut(|decoded| {
        if decoded.len() >= MAX_CACHED_ICONS {
            decoded.clear();
        }
        decoded.insert(icon_uri, texture.clone());
    });
    Some(texture)
}

impl FileCell {
    /// Shows the art again, then looks up `item`'s custom icon and, if it
    /// has none, a folder's picture (PROP-037): on the folder on a tile of
    /// Medium icons or larger, in the icon's place on smaller ones and in
    /// rows; or, where `previews` allow one, its preview (VIEW-057).
    /// A lookup still running for the item shown before is dropped.
    pub(crate) fn look_up_picture(&self, item: &FileItem, previews: PreviewPolicy) {
        self.cancel_picture_lookup();
        let imp = self.imp();
        // A wait for the item shown before ends with it.
        imp.icon_wait.set(imp.icon_wait.get().wrapping_add(1));
        imp.custom_icon.set_visible(false);
        imp.folder_picture.show_texture(None);
        imp.image.set_opacity(1.0);
        let entry = item.entry();
        let uri = entry.uri.clone();
        let pixels = imp.icon_size.get() * self.scale_factor().max(1);
        let preview = previews.request(entry, pixels);
        if preview.is_some() {
            thumbnails::trace::event("bind", &uri);
        }
        // A preview shown lately is drawn on this very frame, as Windows
        // draws a thumbnail it has cached; the lookup below still runs, as
        // the item may have a custom icon since.
        let shown_at_once = preview
            .as_ref()
            .and_then(thumbnails::kept_preview)
            .filter(is_big_enough_to_see);
        if let Some(texture) = &shown_at_once {
            thumbnails::trace::event("shown-at-bind", &uri);
            self.show_picture(texture);
        } else if preview.as_ref().is_some_and(thumbnails::probably_cached) {
            // The cache holds its thumbnail, which takes a moment to read:
            // the tile waits for it without its type icon, rather than
            // showing an icon replaced a moment later, for a moment at
            // most.
            self.hide_type_icon_for_a_moment(&uri);
        }
        let kind_picture = crate::icons::file_icons::picture_for(entry);
        if !uri.starts_with("file:") && preview.is_none() && kind_picture.is_none() {
            return;
        }
        let is_folder = entry.is_dir;
        let on_the_folder = imp.is_tile.get() && imp.icon_size.get() >= SMALLEST_PICTURE_ICON;
        let cell = self.downgrade();
        // The preview is read while the item's own pictures are looked up,
        // which takes a while for the many tiles of one layout: a tile
        // shows its thumbnail as soon as it is read, and a custom icon
        // found after it takes its place.
        let own_found = Rc::new(Cell::new(false));
        let reading = preview.filter(|_| shown_at_once.is_none()).map(|request| {
            let (cell, own_found, uri) = (self.downgrade(), Rc::clone(&own_found), uri.clone());
            Reading(Some(glib::spawn_future_local(async move {
                let texture = thumbnails::preview(request).await.filter(is_big_enough_to_see)?;
                if let Some(cell) = cell.upgrade().filter(|_| !own_found.get()) {
                    thumbnails::trace::event("shown", &uri);
                    cell.show_picture(&texture);
                }
                Some(texture)
            })))
        });
        let lookup = glib::spawn_future_local(async move {
            let own = own_pictures(&uri).await;
            thumbnails::trace::event("own-pictures", &uri);
            own_found.set(own.icon.is_some() || (is_folder && own.folder_picture.is_some()));
            let mut texture = match own.icon {
                Some(icon) => icon_texture(icon).await,
                None => None,
            };
            // A folder's picture sits on the folder at Medium icons and
            // larger, and takes the small icon's place in Details, List
            // and Small icons, as Windows draws it there.
            if let Some(picture) = own.folder_picture.filter(|_| texture.is_none() && is_folder) {
                if on_the_folder {
                    if let (Some(cell), Some(picture)) = (cell.upgrade(), icon_texture(picture).await) {
                        cell.imp().folder_picture.show_texture(Some(&picture));
                        cell.imp().picture_lookup.take();
                    }
                    return;
                }
                texture = icon_texture(picture).await;
            }
            if texture.is_none() {
                if let Some(shown) = shown_at_once {
                    // Already shown, and still the item's picture.
                    if let Some(cell) = cell.upgrade() {
                        cell.imp().picture_lookup.take();
                    }
                    drop(shown);
                    return;
                }
                if let Some(reading) = reading {
                    // Shown as soon as it was read.
                    let read = reading.finish().await;
                    if let Some(cell) = cell.upgrade() {
                        if read.is_none() {
                            // No picture after all: the type icon it waited
                            // without.
                            cell.imp().image.set_opacity(1.0);
                        }
                        if read.is_some() || kind_picture.is_none() {
                            cell.imp().picture_lookup.take();
                            return;
                        }
                    }
                }
            }
            if let Some(picture) = kind_picture.filter(|_| texture.is_none()) {
                texture = icon_texture(picture).await;
            }
            let Some(cell) = cell.upgrade() else {
                return;
            };
            if let Some(texture) = texture {
                thumbnails::trace::event("shown", &uri);
                cell.show_picture(&texture);
            } else {
                // No picture after all: the type icon it waited without.
                cell.imp().image.set_opacity(1.0);
            }
            cell.imp().picture_lookup.take();
        });
        imp.picture_lookup.replace(Some(lookup));
    }

    /// Shows `texture` in place of the type icon.
    fn show_picture(&self, texture: &gdk::Texture) {
        let imp = self.imp();
        imp.custom_icon.set_paintable(Some(texture));
        imp.custom_icon.set_visible(true);
        imp.image.set_opacity(0.0);
    }

    /// Hides the type icon while a thumbnail the cache holds is read, and
    /// shows it again after [`ICON_WAIT`] if the thumbnail has not come,
    /// as the disk may be busy.
    fn hide_type_icon_for_a_moment(&self, uri: &str) {
        let imp = self.imp();
        imp.image.set_opacity(0.0);
        thumbnails::trace::event("icon-held", uri);
        let generation = imp.icon_wait.get().wrapping_add(1);
        imp.icon_wait.set(generation);
        let cell = self.downgrade();
        let uri = uri.to_owned();
        glib::timeout_add_local_once(ICON_WAIT, move || {
            let Some(cell) = cell.upgrade() else {
                return;
            };
            let imp = cell.imp();
            if imp.icon_wait.get() == generation && !imp.custom_icon.is_visible() {
                thumbnails::trace::event("icon-revealed", &uri);
                imp.image.set_opacity(1.0);
            }
        });
    }

    /// Drops the lookup of a custom icon or preview still running, as for
    /// a row GTK unbinds when it scrolls away.
    pub(crate) fn cancel_picture_lookup(&self) {
        if let Some(lookup) = self.imp().picture_lookup.take() {
            lookup.abort();
        }
    }

    /// Whether the cell shows its folder's picture on the folder, for
    /// tests.
    #[cfg(test)]
    pub(crate) fn shows_folder_picture(&self) -> bool {
        self.imp().folder_picture.shows_picture()
    }

    /// Whether the cell shows a custom icon or a preview, for tests.
    #[cfg(test)]
    pub(crate) fn shows_custom_icon(&self) -> bool {
        self.imp().custom_icon.is_visible()
    }
}

/// A preview being read for a cell; stopped when the cell no longer
/// wants it, as when it is bound to another item.
struct Reading(Option<glib::JoinHandle<Option<gdk::Texture>>>);

impl Reading {
    /// The preview, once read.
    async fn finish(mut self) -> Option<gdk::Texture> {
        self.0.take()?.await.ok().flatten()
    }
}

impl Drop for Reading {
    fn drop(&mut self) {
        if let Some(reading) = self.0.take() {
            reading.abort();
        }
    }
}

/// Fewest pixels across the shorter side of a thumbnail the cell shows.
const THINNEST_THUMBNAIL: i32 = 4;
/// Fewest pixels across the longer side of a thumbnail the cell shows.
const SMALLEST_THUMBNAIL: i32 = 8;

/// Whether a picture's thumbnail `texture` is large enough to stand for
/// it. One smaller, such as the 1×1 white picture New > Bitmap image
/// makes, would leave its row looking iconless on a light background, so
/// the item keeps its type's icon instead.
fn is_big_enough_to_see(texture: &gdk::Texture) -> bool {
    let (width, height) = (texture.width(), texture.height());
    width.min(height) >= THINNEST_THUMBNAIL && width.max(height) >= SMALLEST_THUMBNAIL
}

/// The pictures a local item names for itself: its custom icon and, for
/// a folder, the picture on its icon.
#[derive(Debug, Default)]
struct OwnPictures {
    icon: Option<String>,
    folder_picture: Option<String>,
}

/// The custom icon and folder picture of the local item at `uri`, read
/// in one query.
async fn own_pictures(uri: &str) -> OwnPictures {
    if !uri.starts_with("file:") {
        return OwnPictures::default();
    }
    let file = gio::File::for_uri(uri);
    let Ok(info) = file
        .query_info_future(
            &format!("{CUSTOM_ICON},{FOLDER_PICTURE}"),
            gio::FileQueryInfoFlags::NONE,
            glib::Priority::LOW,
        )
        .await
    else {
        return OwnPictures::default();
    };
    let named = |key: &str| {
        info.attribute_string(key)
            .filter(|picture| !picture.is_empty())
            .map(|picture| picture.to_string())
    };
    OwnPictures {
        icon: named(CUSTOM_ICON),
        folder_picture: named(FOLDER_PICTURE),
    }
}

/// The picture at `uri` at icon size, if it is a local image of a
/// sensible size.
fn decode(uri: &str) -> Option<gdk::Texture> {
    let path = gio::File::for_uri(uri).path()?;
    let size = std::fs::metadata(&path).ok()?.len();
    if size > MAX_ICON_BYTES {
        return None;
    }
    let file = gio::File::for_path(&path);
    let decoded =
        picture_guard::decode_to_fit(&file, ICON_PIXELS, ICON_PIXELS, Limits::DISPLAY, None).ok()?;
    Some(gdk::Texture::for_pixbuf(&decoded.pixbuf))
}
