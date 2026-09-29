//! The chosen folders and the last Scan's Plan: remembered folders, Scan on open and on Rescan,
//! and Builds that apply the Plan kept in memory.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use futures_channel::oneshot;
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, SharedString};

use crate::build::{self, Report};
use crate::scan::{self, Plan};
use crate::settings::{self, Settings};
use crate::view::Table;
use crate::{AppWindow, format};

const STOPPED: &str = "The work stopped unexpectedly.";

pub struct Session {
    table: Rc<Table>,
    /// Filled by the last successful Scan; a Build takes it and hands back the published Plan.
    plan: RefCell<Option<Plan>>,
}

impl Session {
    /// Wires the folder pickers, Rescan and Build, then restores the saved folders and scans them.
    /// The installed callbacks own the session from then on.
    pub fn start(ui: &AppWindow, table: Rc<Table>) {
        let session = Rc::new(Self { table, plan: RefCell::default() });
        let saved = settings::load();
        ui.set_source_path(saved.source.into());
        ui.set_update_tree_path(saved.update_tree.into());

        ui.on_pick_source(session.callback(ui, |session, ui| {
            if let Some(path) = pick_folder(ui, "Choose the Source folder", &ui.get_source_path()) {
                ui.set_source_path(path);
                session.folders_changed(ui);
            }
        }));
        ui.on_pick_update_tree(session.callback(ui, |session, ui| {
            if let Some(path) = pick_folder(ui, "Choose the Update tree folder", &ui.get_update_tree_path()) {
                ui.set_update_tree_path(path);
                session.folders_changed(ui);
            }
        }));
        ui.on_rescan(session.callback(ui, |session, ui| session.scan(ui)));
        ui.on_build(session.callback(ui, |session, ui| session.build(ui)));

        // Scans once the event loop runs, so the window shows up before the hashing starts.
        let first_scan = session.callback(ui, |session, ui| session.scan(ui));
        slint::Timer::single_shot(Duration::ZERO, first_scan);
    }

    fn callback(self: &Rc<Self>, ui: &AppWindow, action: fn(&Rc<Self>, &AppWindow)) -> impl Fn() + 'static {
        let (session, ui) = (self.clone(), ui.as_weak());
        move || {
            if let Some(ui) = ui.upgrade() {
                action(&session, &ui);
            }
        }
    }

    fn folders_changed(self: &Rc<Self>, ui: &AppWindow) {
        let saved = Settings { source: ui.get_source_path().into(), update_tree: ui.get_update_tree_path().into() };
        if let Err(error) = settings::save(&saved) {
            show_error(ui, "Folders not saved", &format!("The chosen folders will not be remembered: {error}"));
        }
        self.scan(ui);
    }

    /// Hashes the Source on a worker thread and keeps the resulting Plan for the next Build.
    fn scan(self: &Rc<Self>, ui: &AppWindow) {
        let (source, update_tree) = (ui.get_source_path(), ui.get_update_tree_path());
        if source.is_empty() || update_tree.is_empty() || busy(ui) {
            return;
        }
        self.plan.replace(None);
        self.table.clear(ui);
        ui.set_can_build(false);
        ui.set_scan_progress(0.0);
        ui.set_scanning(true);

        let progress = progress_reporter(ui, AppWindow::set_scan_progress, AppWindow::get_scan_progress);
        let (source, update_tree) = (PathBuf::from(source.as_str()), PathBuf::from(update_tree.as_str()));
        let job = in_background(move || scan::run(&source, &update_tree, &progress));
        self.when_done(ui, job, |session, ui, result| {
            ui.set_scanning(false);
            match result {
                Ok(Ok(plan)) => {
                    let note = scan_note(&plan);
                    session.show(ui, plan, &note);
                }
                Ok(Err(error)) => show_error(ui, "Scan failed", &error.to_string()),
                Err(_) => show_error(ui, "Scan failed", STOPPED),
            }
        });
    }

    /// Builds the Plan in memory; files edited since the Scan need a Rescan first.
    fn build(self: &Rc<Self>, ui: &AppWindow) {
        if busy(ui) {
            return;
        }
        let Some(plan) = self.plan.take() else { return };
        ui.set_build_progress(0.0);
        ui.set_building(true);

        let progress = progress_reporter(ui, AppWindow::set_build_progress, AppWindow::get_build_progress);
        let job = in_background(move || {
            let result = build::run(&plan, &progress);
            (plan, result)
        });
        self.when_done(ui, job, |session, ui, outcome| {
            ui.set_building(false);
            match outcome {
                Ok((plan, Ok(report))) => {
                    let note = build_note(&report);
                    session.show(ui, plan.into_published(report.manifest), &note);
                }
                Ok((plan, Err(error))) => {
                    session.plan.replace(Some(plan));
                    show_error(ui, "Build failed", &error.to_string());
                }
                Err(_) => {
                    // The Plan went down with the worker; only a Rescan can make a new one.
                    ui.set_can_build(false);
                    show_error(ui, "Build failed", STOPPED);
                }
            }
        });
    }

    fn show(&self, ui: &AppWindow, plan: Plan, note: &str) {
        self.table.show(ui, plan.files.clone(), note);
        ui.set_can_build(plan.has_work());
        self.plan.replace(Some(plan));
    }

    /// Runs `finish` on the UI thread once `job` completes, if the window still exists.
    fn when_done<T: 'static>(
        self: &Rc<Self>,
        ui: &AppWindow,
        job: impl Future<Output = Result<T, oneshot::Canceled>> + 'static,
        finish: fn(&Rc<Self>, &AppWindow, Result<T, oneshot::Canceled>),
    ) {
        let (session, ui) = (self.clone(), ui.as_weak());
        slint::spawn_local(async move {
            let result = job.await;
            if let Some(ui) = ui.upgrade() {
                finish(&session, &ui, result);
            }
        })
        .expect("callbacks run inside the Slint event loop");
    }
}

fn busy(ui: &AppWindow) -> bool {
    ui.get_scanning() || ui.get_building()
}

/// Runs `work` on a new thread; resolves with its result, or `Canceled` if it panicked.
fn in_background<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> impl Future<Output = Result<T, oneshot::Canceled>> {
    let (sender, receiver) = oneshot::channel();
    std::thread::spawn(move || {
        // The receiver only disappears when the app is closing.
        let _ = sender.send(work());
    });
    receiver
}

/// Progress callback for worker threads, posting at most one UI update per whole percent.
fn progress_reporter(
    ui: &AppWindow,
    set: fn(&AppWindow, f32),
    get: fn(&AppWindow) -> f32,
) -> impl Fn(u64, u64) + Send + Sync + 'static {
    let (ui, reported) = (ui.as_weak(), AtomicU32::new(0));
    move |done, total| {
        let percent = (done * 100).checked_div(total).map_or(100, |percent| percent as u32);
        if reported.fetch_max(percent, Ordering::Relaxed) < percent {
            let _ = ui.upgrade_in_event_loop(move |ui| {
                // Workers may post out of order; progress only moves forward.
                set(&ui, get(&ui).max(percent as f32 / 100.0));
            });
        }
    }
}

/// Summary subtitle after a Scan: what the next Build would publish. Short enough for one line.
fn scan_note(plan: &Plan) -> String {
    if !plan.has_work() {
        return format!("Up to date · Manifest v{}", plan.next_version());
    }
    let mut note = format!("Next: Manifest v{}", plan.next_version());
    if !plan.removed.is_empty() {
        note.push_str(&format!(" · {} to remove", format::count(plan.removed.len() as u64)));
    }
    note
}

/// Summary subtitle after a Build: the Manifest Launchers now read and what was written.
fn build_note(report: &Report) -> String {
    let verb = if report.manifest_written { "Published" } else { "Kept" };
    let mut note = format!("{verb} Manifest v{}", report.manifest.version);
    if report.rezipped > 0 {
        note.push_str(&format!(" · {} re-zipped", format::count(report.rezipped)));
    }
    if report.removed_archives > 0 {
        note.push_str(&format!(" · {} removed", format::count(report.removed_archives)));
    }
    note
}

/// Native folder dialog opened at `current`, modal to the app window; `None` when cancelled.
fn pick_folder(ui: &AppWindow, title: &str, current: &str) -> Option<SharedString> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    if !current.is_empty() {
        dialog = dialog.set_directory(current);
    }
    let folder = ui.window().with_winit_window(|window| dialog.set_parent(window).pick_folder()).flatten()?;
    // Shown with `/` separators, like Manifest paths; Windows accepts both.
    Some(folder.to_string_lossy().replace('\\', "/").into())
}

pub fn show_error(ui: &AppWindow, title: &str, message: &str) {
    let dialog = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(title)
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok);
    ui.window().with_winit_window(|window| dialog.set_parent(window).show());
}
