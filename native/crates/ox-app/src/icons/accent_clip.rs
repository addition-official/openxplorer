// SPDX-License-Identifier: AGPL-3.0-only
//! A box of a fixed size that shows its child moved up and left by an
//! offset and cut off at its edges: the accent part of a command glyph,
//! the same glyph clipped to a rectangle of it ([`super::accented_image`]).
//! Unlike a `GtkFixed`, it never grows to fit its child.

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use std::cell::Cell;

    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    /// Private state of [`super::AccentClip`].
    #[derive(Default)]
    pub(crate) struct AccentClip {
        /// The box's width and height.
        pub(super) size: Cell<(i32, i32)>,
        /// How far up and left the child is moved.
        pub(super) offset: Cell<(i32, i32)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AccentClip {
        const NAME: &'static str = "OxAccentClip";
        type Type = super::AccentClip;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for AccentClip {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for AccentClip {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let (width, height) = self.size.get();
            let edge = if orientation == gtk::Orientation::Horizontal {
                width
            } else {
                height
            };
            (edge, edge, -1, -1)
        }

        fn size_allocate(&self, _width: i32, _height: i32, baseline: i32) {
            let (x, y) = self.offset.get();
            if let Some(child) = self.obj().first_child() {
                let (_, natural) = child.preferred_size();
                // Icon offsets are a few pixels, exact in an `f32`.
                let shift = |pixels: i32| f32::from(i16::try_from(-pixels).unwrap_or(0));
                let transform =
                    gtk::gsk::Transform::new().translate(&gtk::graphene::Point::new(shift(x), shift(y)));
                child.allocate(natural.width(), natural.height(), baseline, Some(transform));
            }
        }
    }
}

glib::wrapper! {
    /// A fixed-size box showing its child moved and clipped.
    pub(crate) struct AccentClip(ObjectSubclass<imp::AccentClip>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl AccentClip {
    /// A `width` by `height` box showing `child` moved `x` left and `y` up.
    pub(crate) fn new(
        child: &impl IsA<gtk::Widget>,
        (x, y): (i32, i32),
        (width, height): (i32, i32),
    ) -> Self {
        let clip: Self = glib::Object::builder()
            .property("overflow", gtk::Overflow::Hidden)
            .property("can-target", false)
            .property("accessible-role", gtk::AccessibleRole::Presentation)
            .build();
        clip.imp().size.set((width.max(0), height.max(0)));
        clip.imp().offset.set((x, y));
        child.set_parent(&clip);
        clip
    }
}
