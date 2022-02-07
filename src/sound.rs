
use kira::sound::static_sound::StaticSoundHandle;
use kira::{

    manager::AudioManager,

};

use kira_cpal::CpalBackend;
#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::hash::Hasher;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Error, Result};

/// The playlist
pub type SoundQueue = Vec<MetaSound>;

pub trait Playlist {
    fn contains(&self, _sound: &MetaSound) -> bool {
        unimplemented!()
    }
    fn to_index(&self, _sound: &MetaSound) -> Option<usize> {
        unimplemented!()
    }
}

impl Playlist for SoundQueue {
    fn contains(&self, sound: &MetaSound) -> bool {
        self.iter().filter(|s| *s == sound).count() > 0
    }

    fn to_index(&self, sound: &MetaSound) -> Option<usize> {
        for (i, s) in self.iter().enumerate() {
            if s == sound {
                return Some(i);
            }
        }
        None
    }
}

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Default)]
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
    pub soundhandle: Option<StaticSoundHandle>,
    #[serde(skip)]
    pub instancehandle: Option<InstanceHandle>,
    pub bookmarks: Vec<f64>,
}

impl PartialEq for MetaSound {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for MetaSound {}

impl Hash for MetaSound {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.path.hash(hasher);
    }
}

impl MetaSound {
    pub fn load_tag(&self) -> Result<Self> {
        let tag = audiotags::Tag::new().read_from_path(&self.path)?;
        let title = tag.title().ok_or(anyhow!("Can't read title"))?;
        let artist = tag.artist().ok_or(anyhow!("Can't read artist"))?;
        let album = tag.album().ok_or(anyhow!("Can't read album"))?.title;
        Ok(Self {
            name: format!("{} - {} | {}", artist, title, album),
            ..self.clone()
        })
    }

    pub fn with_path<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            path: path.as_ref().into(),
            name: nice_name(path.as_ref()),
            ..self.clone()
        }
    }

    pub fn is_supported(&self) -> bool {
        match self.path.extension() {
            Some(ext) => match ext.to_string_lossy().to_string().as_str() {
                "mp3" | "m4a" | "ogg" | "flac" | "wav" => true,
                _ => false,
            },
            None => false,
        }
    }

    // Tries to load metadata and tags, but does not fail.
    pub fn try_meta(&self) -> Self {
        match self.load_tag() {
            Ok(s_id_tag) => s_id_tag,
            Err(_) => self.clone(),
        }
    }

    pub fn load(&self, manager: &mut AudioManager<CpalBackend>) -> Result<StaticSoundHandle, Error> {
        manager
            .load_sound(&self.path, SoundSettings::default())
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn load_soundhandle(&self, manager: &mut AudioManager<CpalBackend>) -> Self {
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
            .ok_or(anyhow!("Sound handle is None. Is this sound loaded?"))?;
        let instancehandle = soundhandle.play(InstanceSettings::new())?;
        self.instancehandle = Some(instancehandle);
        Ok(())
    }

    pub fn play_load_mut(&mut self, manager: &mut AudioManager<CpalBackend>) -> Result<()> {
        self.soundhandle = self.load(manager).ok();
        if let Some(handle) = &self.soundhandle {
            self.duration = Duration::from_secs_f64(handle.duration());
        }
        self.play()?;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(h) = &mut self.soundhandle {
            let _ = h.stop(StopInstanceSettings::new());
        }
    }
}

pub fn nice_name(p: &Path) -> String {
    p.file_name()
        .unwrap_or(OsStr::new("no path"))
        .to_string_lossy()
        .replace("_", " ")
        .replace("-", " ")
}
