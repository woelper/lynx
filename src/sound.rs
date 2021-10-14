use kira::instance::{InstanceSettings, InstanceState, StopInstanceSettings};
use kira::{
    instance::{handle::InstanceHandle, PauseInstanceSettings, ResumeInstanceSettings},
    manager::AudioManager,
    sound::{handle::SoundHandle, SoundSettings},
};

#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Error, Result};

/// The playlist
pub type SoundQueue = Vec<MetaSound>;

pub trait Playlist {
    fn contains(&self, sound: &MetaSound) -> bool {
        unimplemented!()
    }
}

impl Playlist for SoundQueue {
    fn contains(&self, sound: &MetaSound) -> bool {
        self.iter().filter(|s| s.name == sound.name).count() > 0
    }
}

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Default)]
/// A high-level sound
pub struct MetaSound {
    /// Location of sound
    pub path: PathBuf,
    /// Nice name of sound
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Duration,
    pub looped: bool,
    #[serde(skip)]
    pub soundhandle: Option<SoundHandle>,
    #[serde(skip)]
    pub instancehandle: Option<Arc<InstanceHandle>>,
}

impl MetaSound {
    pub fn load_tag(&self) -> Result<Self> {
        let f = taglib::File::new(&self.path).map_err(|e| anyhow!("{:?}", e))?;
        let tag = f.tag().map_err(|e| anyhow!("{:?}", e))?;
        let title = tag.title().ok_or(anyhow!("Can't read title"))?;
        let artist = tag.artist().ok_or(anyhow!("Can't read artist"))?;
        let album = tag.album().ok_or(anyhow!("Can't read album"))?;
        Ok(Self {
            name: format!("{} - {} | {}", artist, title, album),
            ..self.clone()
        })
    }

    pub fn with_path<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            name: nice_name(&path.as_ref()),
            ..self.clone()
        }
    }

    // Tries to load metadata and tags, but does not fail.
    pub fn try_meta(&self) -> Self {
        match self.load_tag() {
            Ok(s_id_tag) => s_id_tag,
            Err(_) => self.clone(),
        }
    }

    pub fn load(&self, manager: &mut AudioManager) -> Result<SoundHandle, Error> {
        manager
            .load_sound(&self.path, SoundSettings::default())
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn load_soundhandle(&self, manager: &mut AudioManager) -> Self {
        let handle = self.load(manager).ok();
        Self {
            soundhandle: handle,
            ..self.clone()
        }
    }

    pub fn play(&mut self) -> Result<()> {
        let soundhandle = self
            .soundhandle
            .as_mut()
            .ok_or(anyhow!("Sound handle is None"))?;
        let instancehandle = soundhandle.play(InstanceSettings::new())?;
        self.instancehandle = Some(Arc::new(instancehandle));
        Ok(())
    }

    pub fn load_mut(&mut self, manager: &mut AudioManager) -> Result<()> {
        self.soundhandle = self.load(manager).ok();
        if let Some(handle) = &self.soundhandle {
            self.duration = Duration::from_secs_f64(handle.duration());
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(h) = &mut self.soundhandle {
            let _ = h.stop(StopInstanceSettings::new());
        }
    }
}

pub fn nice_name(p: &Path) -> String {
    format!(
        "{}",
        p.file_name()
            .unwrap_or(OsStr::new("no path"))
            .to_string_lossy()
            .replace("_", " ")
            .replace("-", " ")
    )
}
