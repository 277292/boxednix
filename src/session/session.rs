// SPDX-License-Identifier: GPL-3.0-only

use anyhow::anyhow;
use notify::{
    self, event::AccessKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    path::Path,
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{info, Result};

use super::SessionFile;

pub trait WatcherContext {
    fn target(&self) -> &Path;
    fn target_file(&self) -> &Path;
}

pub struct Session {
    watcher: RecommendedWatcher,
    handle: JoinHandle<Result<()>>,
}

impl Session {
    pub fn start(
        session_file: SessionFile,
        on_change: impl Fn(&mut SessionFile) -> Result<()> + Send + Sync + 'static,
    ) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        Ok(Self {
            watcher: Self::watcher(tx, &session_file)?,
            handle: Self::spawn_event_loop(rx, session_file, Box::new(on_change)),
        })
    }

    fn spawn_event_loop(
        rx: Receiver<notify::Result<Event>>,
        mut session_file: SessionFile,
        on_change: Box<dyn Fn(&mut SessionFile) -> Result<()> + Send + Sync + 'static>,
    ) -> JoinHandle<Result<()>> {
        thread::spawn(move || -> Result<()> {
            while let Ok(event_result) = rx.recv() {
                let event = &event_result?;

                let is_access_close = matches!(event.kind, EventKind::Access(AccessKind::Close(_)));
                if !(is_access_close && session_file.exists_in(&event.paths)) {
                    continue;
                }

                on_change(&mut session_file)?;
            }

            Ok(())
        })
    }

    fn watcher<W: WatcherContext>(
        tx: Sender<notify::Result<Event>>,
        ctx: &W,
    ) -> Result<RecommendedWatcher> {
        let mut watcher = notify::recommended_watcher(tx)?;
        watcher.watch(ctx.target(), RecursiveMode::NonRecursive)?;
        info!("Wachting: {:?}", ctx.target_file());
        Ok(watcher)
    }

    pub fn stop(self) -> Result<()> {
        drop(self.watcher);
        info!("Watcher stop");
        self.handle.join().map_err(|e| anyhow!("{:?}", e))?
    }
}
