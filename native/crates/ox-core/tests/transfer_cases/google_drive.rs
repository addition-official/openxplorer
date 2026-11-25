// SPDX-License-Identifier: AGPL-3.0-only
//! Google Drive through the production GIO adapter and the transfer
//! engine, against a simulated `GVfs` Google backend (no account, no
//! network; see `transfer_support/google_drive.rs`): copies and moves in
//! and out, same titles, Docs, cancellation, a full drive, a lost
//! connection, interrupted uploads, and the read-only default (DEV-013).

use std::fs;
use std::sync::{Arc, Mutex};

use ox_core::location::cloud_writes::{set_google_drive_writable, switch_lock};
use ox_core::transfer::{Cancellation, ConflictPolicy, Node, Operation, TransferError, TransferResult};

use crate::transfer_support::google_drive::{DriveCall, FakeDrive, ItemKind, UploadFault, MY_DRIVE_ID};
use crate::transfer_support::{file_uri, gio_engine, temporary_folder};

/// Turns the switch to `on` for one test, and off again afterwards.
struct Switch(#[allow(dead_code)] std::sync::MutexGuard<'static, ()>);

impl Switch {
    fn set(on: bool) -> Self {
        let lock = switch_lock();
        set_google_drive_writable(on);
        Self(lock)
    }
}

impl Drop for Switch {
    fn drop(&mut self) {
        set_google_drive_writable(false);
    }
}

/// Runs `operation` on `uris` with the production adapter. A run refused
/// before its first item, as for a full drive or a destination that cannot
/// be reached, reports its one error as the app shows it.
fn run(operation: Operation<'_>, uris: &[String]) -> TransferResult {
    outcome(gio_engine().run(operation, uris, &Cancellation::new()))
}

/// A run's result, or its refusal as its one error.
fn outcome(run: Result<TransferResult, TransferError>) -> TransferResult {
    run.unwrap_or_else(|refusal| TransferResult {
        errors: vec![refusal.to_string()],
        ..TransferResult::default()
    })
}

fn copy_into(folder: &str, policy: ConflictPolicy) -> Operation<'_> {
    Operation::Copy {
        destination_folder: folder,
        policy,
    }
}

fn move_into(folder: &str, policy: ConflictPolicy) -> Operation<'_> {
    Operation::Move {
        destination_folder: folder,
        policy,
    }
}

/// True when nothing on the drive has the engine's private staging title.
fn no_staging_left(drive: &FakeDrive) -> bool {
    drive
        .all_items()
        .iter()
        .all(|item| !ox_core::transfer::is_own_staging_name(&item.title))
}

/// A file copied from this PC lands under its own title, with every byte,
/// and nothing private is left: built under a staging title beside it,
/// then given its title by a rename within the folder.
///
/// parity: DEV-013
#[test]
fn a_copy_uploads_under_its_title_through_a_rename() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let local = temporary_folder();
    let source = local.path().join("Report.pdf");
    fs::write(&source, b"report bytes").unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );

    assert!(result.errors.is_empty(), "{result:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["Report.pdf"]);
    let id = drive.id_of(MY_DRIVE_ID, "Report.pdf");
    assert_eq!(drive.bytes(&id), b"report bytes");
    assert!(source.exists(), "a copy keeps the source");
    assert!(no_staging_left(&drive));
    let calls = drive.calls();
    assert!(
        matches!(&calls[..], [DriveCall::Upload(_), DriveCall::Rename { to, .. }] if to == "Report.pdf"),
        "{calls:?}"
    );
    // Where it landed is its address by ID, not by title.
    assert_eq!(result.landed[0].destination, drive.uri_of(&id));
}

/// A folder tree is built under a staging title, filled, and given its
/// title only when complete.
///
/// parity: DEV-013
#[test]
fn a_folder_tree_is_copied_in_and_published_whole() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let local = temporary_folder();
    let tree = local.path().join("Photos");
    fs::create_dir_all(tree.join("2026/Summer")).unwrap();
    fs::write(tree.join("index.txt"), b"list").unwrap();
    fs::write(tree.join("2026/Summer/beach.jpg"), b"jpeg").unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&tree)],
    );

    assert!(result.errors.is_empty(), "{result:?}");
    let photos = drive.id_of(MY_DRIVE_ID, "Photos");
    assert_eq!(drive.titles(&photos), ["2026", "index.txt"]);
    let year = drive.id_of(&photos, "2026");
    let summer = drive.id_of(&year, "Summer");
    let beach = drive.id_of(&summer, "beach.jpg");
    assert_eq!(drive.bytes(&beach), b"jpeg");
    assert!(no_staging_left(&drive));
}

/// Copies off the drive take `GVfs`'s copy names: two items with one
/// title stay two files, and a title with "/" becomes a valid name. This
/// works with changes to Google Drive off, as before.
///
/// parity: DEV-013
#[test]
fn copies_off_the_drive_keep_same_titled_files_apart() {
    let _switch = Switch::set(false);
    let drive = FakeDrive::new();
    let first = drive.add_file(MY_DRIVE_ID, "Notes.txt", b"first");
    let second = drive.add_file(MY_DRIVE_ID, "Notes.txt", b"second");
    let slashed = drive.add_file(MY_DRIVE_ID, "Q1/Q2 plan.txt", b"plan");
    let local = temporary_folder();

    let uris = [
        drive.uri_of(&first),
        drive.uri_of(&second),
        drive.uri_of(&slashed),
    ];
    let result = run(copy_into(&file_uri(local.path()), ConflictPolicy::Skip), &uris);

    assert!(result.errors.is_empty(), "{result:?}");
    let mut names: Vec<String> = fs::read_dir(local.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let second_name = format!("Notes - {second}.txt");
    assert_eq!(names, [second_name.as_str(), "Notes.txt", "Q1-Q2 plan.txt"]);
    assert_eq!(fs::read(local.path().join("Notes.txt")).unwrap(), b"first");
    assert_eq!(fs::read(local.path().join(&second_name)).unwrap(), b"second");
    assert!(drive.calls().is_empty(), "nothing on the drive changed");
}

/// A title the folder already has is a name conflict, as on a local disk:
/// Skip leaves it, Keep both picks a free title, and nothing ever makes a
/// second item with the same title.
///
/// parity: DEV-013
#[test]
fn a_taken_title_is_a_conflict_like_a_taken_name() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.add_file(MY_DRIVE_ID, "a.txt", b"old");
    let local = temporary_folder();
    let source = local.path().join("a.txt");
    fs::write(&source, b"new").unwrap();

    let skipped = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );
    assert_eq!(skipped.skipped.len(), 1, "{skipped:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["a.txt"]);

    let kept = run(
        copy_into(&drive.my_drive(), ConflictPolicy::KeepBoth),
        &[file_uri(&source)],
    );
    assert!(kept.errors.is_empty(), "{kept:?}");
    let titles = drive.titles(MY_DRIVE_ID);
    assert_eq!(titles.len(), 2, "{titles:?}");
    let other = titles
        .iter()
        .find(|title| *title != "a.txt")
        .expect("a free title");
    assert_eq!(drive.bytes(&drive.id_of(MY_DRIVE_ID, "a.txt")), b"old");
    assert_eq!(drive.bytes(&drive.id_of(MY_DRIVE_ID, other)), b"new");
}

/// Replace installs the new file and removes the old one only after, and
/// never through `GVfs`'s overwriting move, which deletes for good first.
///
/// parity: DEV-013
#[test]
fn replace_keeps_the_old_file_until_the_new_one_is_in_place() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.add_file(MY_DRIVE_ID, "a.txt", b"old");
    let local = temporary_folder();
    let source = local.path().join("a.txt");
    fs::write(&source, b"new").unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Replace),
        &[file_uri(&source)],
    );

    assert!(result.errors.is_empty(), "{result:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["a.txt"]);
    assert_eq!(drive.bytes(&drive.id_of(MY_DRIVE_ID, "a.txt")), b"new");
    assert!(no_staging_left(&drive));
    let calls = drive.calls();
    assert!(
        calls
            .iter()
            .all(|call| !matches!(call, DriveCall::Move { overwrite: true, .. })),
        "{calls:?}"
    );
    // The old file is set aside first, the new one renamed into place,
    // and only then is the old one deleted.
    let renamed_in = calls
        .iter()
        .position(|call| matches!(call, DriveCall::Rename { to, .. } if to == "a.txt"))
        .expect("published");
    let deleted = calls
        .iter()
        .position(|call| matches!(call, DriveCall::Delete(_)))
        .expect("old one removed");
    assert!(renamed_in < deleted, "{calls:?}");
}

/// Replace onto a title two items share is refused: `GVfs` would choose
/// one of them. Nothing changes.
///
/// parity: DEV-013
#[test]
fn replace_onto_a_shared_title_changes_nothing() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let first = drive.add_file(MY_DRIVE_ID, "a.txt", b"one");
    let second = drive.add_file(MY_DRIVE_ID, "a.txt", b"two");
    let local = temporary_folder();
    let source = local.path().join("a.txt");
    fs::write(&source, b"new").unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Replace),
        &[file_uri(&source)],
    );

    assert_eq!(result.errors.len(), 1, "{result:?}");
    assert!(result.errors[0].contains("More than one item"), "{result:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["a.txt", "a.txt"]);
    assert_eq!(drive.bytes(&first), b"one");
    assert_eq!(drive.bytes(&second), b"two");
    assert!(no_staging_left(&drive));
}

/// A move between two folders of the drive is one `GVfs` move: the item
/// keeps its ID and bytes, and is never copied.
///
/// parity: DEV-013
#[test]
fn a_move_within_the_drive_is_one_move() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let from = drive.add_folder(MY_DRIVE_ID, "From");
    let to = drive.add_folder(MY_DRIVE_ID, "To");
    let file = drive.add_file(&from, "a.txt", b"bytes");

    let result = run(
        move_into(&drive.uri_of(&to), ConflictPolicy::Skip),
        &[drive.uri_of(&file)],
    );

    assert!(result.errors.is_empty(), "{result:?}");
    assert!(drive.titles(&from).is_empty());
    assert_eq!(drive.id_of(&to, "a.txt"), file, "the same item, moved");
    assert_eq!(
        drive.calls(),
        [DriveCall::Move {
            title: "a.txt".into(),
            overwrite: false
        }]
    );
}

/// An older `GVfs` cannot move to another folder. Without the user's
/// consent the item stays; with it, the item is copied, checked and only
/// then removed from where it was.
///
/// parity: DEV-013
#[test]
fn a_move_older_gvfs_cannot_do_is_finished_by_copying_only_with_consent() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.refuse_moves_across_folders();
    let from = drive.add_folder(MY_DRIVE_ID, "From");
    let to = drive.add_folder(MY_DRIVE_ID, "To");
    let file = drive.add_file(&from, "a.txt", b"bytes");
    let uris = [drive.uri_of(&file)];
    let destination = drive.uri_of(&to);

    let refused = run(move_into(&destination, ConflictPolicy::Skip), &uris);
    assert_eq!(refused.errors.len(), 1, "{refused:?}");
    assert_eq!(drive.titles(&from), ["a.txt"]);
    assert!(drive.titles(&to).is_empty());

    let asked = Arc::new(Mutex::new(0));
    let counted = Arc::clone(&asked);
    let moved = gio_engine()
        .with_move_by_copying_question(move |_| {
            *counted.lock().unwrap() += 1;
            true
        })
        .run(
            move_into(&destination, ConflictPolicy::Skip),
            &uris,
            &Cancellation::new(),
        )
        .unwrap();
    assert!(moved.errors.is_empty(), "{moved:?}");
    assert_eq!(*asked.lock().unwrap(), 1);
    assert!(drive.titles(&from).is_empty());
    assert_eq!(drive.bytes(&drive.id_of(&to, "a.txt")), b"bytes");
    assert!(no_staging_left(&drive));
}

/// Moves in and out of the drive cross file systems: copied, checked,
/// then the source removed, once the user agreed.
///
/// parity: DEV-013
#[test]
fn moves_in_and_out_copy_then_remove_the_source() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let local = temporary_folder();
    let source = local.path().join("in.txt");
    fs::write(&source, b"to drive").unwrap();
    let out = drive.add_file(MY_DRIVE_ID, "out.txt", b"to pc");
    let engine = || gio_engine().with_move_by_copying_question(|_| true);

    let moved_in = engine()
        .run(
            move_into(&drive.my_drive(), ConflictPolicy::Skip),
            &[file_uri(&source)],
            &Cancellation::new(),
        )
        .unwrap();
    let moved_out = engine()
        .run(
            move_into(&file_uri(local.path()), ConflictPolicy::Skip),
            &[drive.uri_of(&out)],
            &Cancellation::new(),
        )
        .unwrap();

    assert!(
        moved_in.errors.is_empty() && moved_out.errors.is_empty(),
        "{moved_in:?} {moved_out:?}"
    );
    assert!(!source.exists());
    assert_eq!(drive.titles(MY_DRIVE_ID), ["in.txt"]);
    assert_eq!(fs::read(local.path().join("out.txt")).unwrap(), b"to pc");
}

/// `GVfs` has no Trash for Google Drive: Trash refuses and keeps the item
/// (never a silent permanent delete), and a permanent delete removes files
/// and whole folders.
///
/// parity: DEV-013
#[test]
fn trash_is_refused_and_delete_is_permanent() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let file = drive.add_file(MY_DRIVE_ID, "a.txt", b"a");
    let folder = drive.add_folder(MY_DRIVE_ID, "Folder");
    drive.add_file(&folder, "inside.txt", b"i");

    assert_eq!(drive.node(&file).can_trash(None), Ok(false));
    let trashed = run(Operation::Trash, &[drive.uri_of(&file)]);
    assert_eq!(trashed.errors.len(), 1, "{trashed:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["Folder", "a.txt"]);

    let deleted = run(Operation::Delete, &[drive.uri_of(&file), drive.uri_of(&folder)]);
    assert!(deleted.errors.is_empty(), "{deleted:?}");
    assert!(drive.titles(MY_DRIVE_ID).is_empty());
}

/// A Docs, Sheets or Slides file is refused with a message that says why,
/// and nothing is written for it, not even an empty file. The rest of a
/// folder copies.
///
/// parity: DEV-013
#[test]
fn a_google_doc_is_refused_and_nothing_is_written_for_it() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let doc = drive.add_document(MY_DRIVE_ID, "Budget");
    let folder = drive.add_folder(MY_DRIVE_ID, "Mixed");
    drive.add_document(&folder, "Plan");
    drive.add_file(&folder, "data.csv", b"1,2");
    let local = temporary_folder();
    let pc = file_uri(local.path());

    let result = run(copy_into(&pc, ConflictPolicy::Skip), &[drive.uri_of(&doc)]);

    assert_eq!(result.errors.len(), 1, "{result:?}");
    assert!(
        result.errors[0].contains("Google Docs, Sheets or Slides"),
        "{result:?}"
    );
    assert_eq!(
        fs::read_dir(local.path()).unwrap().count(),
        0,
        "no file, not even an empty one"
    );

    let mixed = run(copy_into(&pc, ConflictPolicy::Skip), &[drive.uri_of(&folder)]);
    assert!(
        mixed.errors.iter().any(|error| error.contains("Plan")),
        "{mixed:?}"
    );
    assert_eq!(fs::read(local.path().join("Mixed/data.csv")).unwrap(), b"1,2");
    assert!(!local.path().join("Mixed/Plan").exists());

    // Within the drive a Doc moves like any item.
    let into = drive.add_folder(MY_DRIVE_ID, "Docs");
    let moved = run(
        move_into(&drive.uri_of(&into), ConflictPolicy::Skip),
        &[drive.uri_of(&doc)],
    );
    assert!(moved.errors.is_empty(), "{moved:?}");
    assert_eq!(drive.children(&into)[0].kind, ItemKind::Document);
}

/// A big upload reports its progress, and Cancel stops it: the partial
/// file `GVfs` leaves is removed, and nothing has the final title.
///
/// parity: DEV-013
#[test]
fn a_cancelled_upload_leaves_nothing_behind() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.set_upload_chunk(64 * 1024);
    let local = temporary_folder();
    let source = local.path().join("video.mkv");
    fs::write(&source, vec![7_u8; 4 * 1024 * 1024]).unwrap();
    let cancel = Cancellation::new();
    let stopper = cancel.clone();
    let reports = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&reports);
    let mut engine = gio_engine().with_progress(move |progress| {
        if let Some(bytes) = progress.bytes {
            seen.lock().unwrap().push(bytes.file_written);
            if bytes.file_written >= 1024 * 1024 {
                stopper.cancel();
            }
        }
    });

    let result = engine
        .run(
            copy_into(&drive.my_drive(), ConflictPolicy::Skip),
            &[file_uri(&source)],
            &cancel,
        )
        .unwrap();

    assert!(result.cancelled, "{result:?}");
    let reports = reports.lock().unwrap();
    assert!(
        reports
            .iter()
            .any(|&written| written > 0 && written < 4 * 1024 * 1024),
        "{reports:?}"
    );
    assert!(drive.titles(MY_DRIVE_ID).is_empty(), "{:?}", drive.all_items());
    assert!(
        drive
            .calls()
            .iter()
            .any(|call| matches!(call, DriveCall::Delete(_))),
        "partial removed"
    );
}

/// An upload the backend reports as done though bytes are missing is
/// never published as a complete file.
///
/// parity: DEV-013
#[test]
fn an_incomplete_upload_is_never_published() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.set_upload_fault(UploadFault::Truncate(10));
    let local = temporary_folder();
    let source = local.path().join("big.bin");
    fs::write(&source, vec![1_u8; 1000]).unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );

    assert_eq!(result.errors.len(), 1, "{result:?}");
    assert!(result.errors[0].contains("incomplete"), "{result:?}");
    assert!(drive.titles(MY_DRIVE_ID).is_empty(), "{:?}", drive.all_items());
}

/// A full drive is said in Windows' words, before anything is written
/// when its free space is known, and when Google refuses the upload.
///
/// parity: DEV-013
#[test]
fn a_full_drive_is_reported_in_windows_words() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    drive.set_quota(100);
    let local = temporary_folder();
    let source = local.path().join("big.bin");
    fs::write(&source, vec![1_u8; 1000]).unwrap();

    let result = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );
    assert!(!result.errors.is_empty(), "{result:?}");
    assert!(
        result.errors[0].contains("Not enough free space on My Drive"),
        "{result:?}"
    );
    assert!(drive.calls().is_empty(), "nothing was uploaded");

    // Google's own refusal, which GVfs reports as "Permission denied".
    let target = drive.node(MY_DRIVE_ID).child("big.bin".as_ref());
    let source_node = ox_core::gio_node::GioNode::new(&file_uri(&source));
    let refused = source_node.copy_file(target.as_ref(), &Cancellation::new(), &mut |_, _| {});
    assert_eq!(
        refused,
        Err(TransferError::NoSpace(
            "There is not enough space on Google Drive. Delete or move files so you have enough space, then try again."
                .into()
        ))
    );
    assert!(drive.titles(MY_DRIVE_ID).is_empty());
}

/// Offline, copies in and out fail with Windows' "The network location
/// cannot be reached." and leave nothing behind; so does a connection
/// lost halfway through an upload.
///
/// parity: DEV-013
#[test]
fn offline_errors_are_reported_in_windows_words() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let on_drive = drive.add_file(MY_DRIVE_ID, "a.txt", b"a");
    let local = temporary_folder();
    let source = local.path().join("b.txt");
    fs::write(&source, vec![2_u8; 300 * 1024]).unwrap();
    let unreachable = "The network location cannot be reached.";

    drive.set_upload_chunk(16 * 1024);
    drive.set_upload_fault(UploadFault::DropAfter(100 * 1024));
    let dropped = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );
    assert!(
        dropped.errors.iter().any(|error| error.contains(unreachable)),
        "{dropped:?}"
    );
    assert_eq!(drive.titles(MY_DRIVE_ID), ["a.txt"], "{:?}", drive.all_items());

    drive.set_upload_fault(UploadFault::None);
    drive.set_offline(true);
    let up = run(
        copy_into(&drive.my_drive(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );
    let down = run(
        copy_into(&file_uri(local.path()), ConflictPolicy::Skip),
        &[drive.uri_of(&on_drive)],
    );
    assert!(
        up.errors.iter().any(|error| error.contains(unreachable)),
        "{up:?}"
    );
    assert!(
        down.errors.iter().any(|error| error.contains(unreachable)),
        "{down:?}"
    );
    assert!(!local.path().join("a.txt").exists());
    drive.set_offline(false);
    assert_eq!(drive.titles(MY_DRIVE_ID), ["a.txt"]);
}

/// With "Allow changes to Google Drive" off, which is the default, the
/// adapter itself refuses every write to the drive, whatever asked for
/// it, and the drive receives no call that changes it.
///
/// parity: DEV-013
#[test]
fn with_the_switch_off_nothing_writes_to_the_drive() {
    let _switch = Switch::set(false);
    let drive = FakeDrive::new();
    let folder = drive.add_folder(MY_DRIVE_ID, "Folder");
    let file = drive.add_file(MY_DRIVE_ID, "a.txt", b"a");
    let local = temporary_folder();
    let source = local.path().join("b.txt");
    fs::write(&source, b"b").unwrap();
    let engine = || gio_engine().with_move_by_copying_question(|_| true);
    let cancel = Cancellation::new();

    let results = [
        run(
            copy_into(&drive.my_drive(), ConflictPolicy::Replace),
            &[file_uri(&source)],
        ),
        engine()
            .run(
                move_into(&drive.my_drive(), ConflictPolicy::Skip),
                &[file_uri(&source)],
                &cancel,
            )
            .unwrap(),
        run(
            move_into(&drive.uri_of(&folder), ConflictPolicy::Skip),
            &[drive.uri_of(&file)],
        ),
        engine()
            .run(
                move_into(&file_uri(local.path()), ConflictPolicy::Skip),
                &[drive.uri_of(&file)],
                &cancel,
            )
            .unwrap(),
        run(Operation::Delete, &[drive.uri_of(&file)]),
        run(Operation::Trash, &[drive.uri_of(&file)]),
    ];
    for result in &results {
        assert_eq!(result.errors.len(), 1, "{result:?}");
    }
    let node = drive.node(&file);
    let refusals = [
        node.delete(),
        node.move_native(drive.node(&folder).child("a.txt".as_ref()).as_ref(), None),
        drive
            .node(MY_DRIVE_ID)
            .child("New folder".as_ref())
            .create_directory(None),
        ox_core::gio_node::rename_on_google_drive(&drive.uri_of(&file), "b.txt", &cancel).map(|_| ()),
    ];
    for refusal in refusals {
        let Err(TransferError::Failed(message)) = refusal else {
            panic!("refused: {refusal:?}");
        };
        assert!(message.contains("Allow changes to Google Drive"), "{message}");
    }
    assert!(drive.calls().is_empty(), "{:?}", drive.calls());
    assert!(source.exists());
    assert_eq!(drive.titles(MY_DRIVE_ID), ["Folder", "a.txt"]);
}

/// The account's top folder, Shared with me and My Drive itself are
/// never changed, even with the switch on.
///
/// parity: DEV-013
#[test]
fn the_drive_top_folders_stay_fixed() {
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let local = temporary_folder();
    let source = local.path().join("b.txt");
    fs::write(&source, b"b").unwrap();

    let into_top = run(
        copy_into(&drive.top(), ConflictPolicy::Skip),
        &[file_uri(&source)],
    );
    let my_drive_deleted = run(Operation::Delete, &[drive.my_drive()]);

    assert_eq!(into_top.errors.len(), 1, "{into_top:?}");
    assert_eq!(my_drive_deleted.errors.len(), 1, "{my_drive_deleted:?}");
    assert!(drive.calls().is_empty(), "{:?}", drive.calls());
}

/// Runs an operation's future to its end on a private main context.
fn block_on<F: std::future::Future>(operation: F) -> F::Output {
    glib::MainContext::new().block_on(operation)
}

/// Rename gives the item a new title and keeps its ID; a title another
/// item has is refused, as a taken name on a local disk; Undo gives the
/// old title back.
///
/// parity: DEV-013
#[test]
fn rename_changes_the_title_and_undo_gives_it_back() {
    use ox_core::ops::{rename_item, undo, OperationContext};
    let _switch = Switch::set(true);
    let drive = FakeDrive::new();
    let file = drive.add_file(MY_DRIVE_ID, "draft.txt", b"d");
    drive.add_file(MY_DRIVE_ID, "taken.txt", b"t");
    let context = OperationContext::default();

    let taken = block_on(rename_item(&drive.uri_of(&file), "taken.txt", &context));
    assert!(taken.is_err(), "{taken:?}");
    assert!(taken.unwrap_err().to_string().contains("already exists"));

    let renamed = block_on(rename_item(&drive.uri_of(&file), "final.txt", &context)).expect("renamed");
    assert_eq!(renamed.uri, drive.uri_of(&file), "the item keeps its ID address");
    assert!(!renamed.is_unchanged());
    assert_eq!(drive.titles(MY_DRIVE_ID), ["final.txt", "taken.txt"]);
    assert_eq!(drive.id_of(MY_DRIVE_ID, "final.txt"), file);

    let record = renamed.undo_record().expect("undoable");
    let undone = block_on(undo(&record, &context, |_| {})).expect("undo runs");
    assert!(undone.errors.is_empty(), "{undone:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["draft.txt", "taken.txt"]);
    assert!(
        drive
            .calls()
            .iter()
            .all(|call| matches!(call, DriveCall::Rename { .. })),
        "{:?}",
        drive.calls()
    );
}

/// New folder and New text document create the item on the drive and
/// give back its address by ID; a taken title is refused, never
/// duplicated; with the switch off nothing is created.
///
/// parity: DEV-013
#[test]
fn new_items_are_created_once_and_named_by_id() {
    use ox_core::location::ItemKind as NewKind;
    use ox_core::ops::{create_item, OperationContext};
    let drive = FakeDrive::new();
    let context = OperationContext::default();
    {
        let _off = Switch::set(false);
        let refused = block_on(create_item(
            &drive.my_drive(),
            "New folder",
            NewKind::Folder,
            &context,
        ));
        assert!(refused.is_err(), "{refused:?}");
        assert!(drive.calls().is_empty());
    }
    let _switch = Switch::set(true);

    let folder = block_on(create_item(
        &drive.my_drive(),
        "New folder",
        NewKind::Folder,
        &context,
    ))
    .expect("folder");
    let text = block_on(create_item(
        &drive.my_drive(),
        "New Text Document.txt",
        NewKind::File,
        &context,
    ))
    .expect("file");
    let again = block_on(create_item(
        &drive.my_drive(),
        "New folder",
        NewKind::Folder,
        &context,
    ));

    assert_eq!(folder.uri, drive.uri_of(&drive.id_of(MY_DRIVE_ID, "New folder")));
    assert_eq!(
        text.uri,
        drive.uri_of(&drive.id_of(MY_DRIVE_ID, "New Text Document.txt"))
    );
    assert!(again.is_err(), "{again:?}");
    assert_eq!(drive.titles(MY_DRIVE_ID), ["New Text Document.txt", "New folder"]);
    assert!(drive
        .bytes(&drive.id_of(MY_DRIVE_ID, "New Text Document.txt"))
        .is_empty());
}
