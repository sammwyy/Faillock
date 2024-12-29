use std::path::PathBuf;

use anyhow::{Context, Result};
use anyhow::{Error, Ok};
use inotify::{EventMask, Inotify, WatchMask};
use log::debug;

use crate::file_reader::FileReader;

pub struct FileWatcher {
    pub reader: FileReader,
    pub inotify: Option<Inotify>,
}

impl FileWatcher {
    pub fn new(file: &PathBuf) -> FileWatcher {
        let reader = FileReader::new(file);
        FileWatcher {
            reader,
            inotify: None,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let notify = Inotify::init()?;
        let path = self.reader.path.clone();
        debug!("Init file watcher for file {:?}", path);
        notify.watches().add(path, WatchMask::MODIFY)?;
        self.inotify = Some(notify);
        Ok(())
    }

    pub fn wait_for_changes(&mut self) -> Result<Option<String>> {
        let mut buffer = [0; 1024];
        let notify = self.inotify.as_mut().unwrap();
        let events = notify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                debug!("File {:?} changed", self.reader.path);
                return Ok(Some(self.reader.read()));
            }
        }

        Ok(None)
    }

    pub fn wait_for_newlines(&mut self) -> Result<Option<Vec<String>>> {
        let mut buffer = [0; 1024];
        let notify = self.inotify.as_mut().unwrap();
        let events = notify.read_events_blocking(&mut buffer)?;

        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                debug!("File {:?} changed", self.reader.path);
                return Ok(Some(self.reader.read_lines()));
            }
        }

        Ok(None)
    }
}
