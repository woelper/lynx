use kira::{
    manager::AudioManager,
    sound::{handle::SoundHandle, SoundSettings},
};
#[cfg(feature = "persistence")]
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
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
        self.iter().filter(|s| s.id == sound.id).count() > 0
    }
}

#[cfg_attr(feature = "persistence", derive(Deserialize, Serialize))]
#[derive(Debug, Clone, Default)]
/// A high-level sound
pub struct MetaSound {
    pub path: PathBuf,
    pub name: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration: Duration,
    pub looped: bool,
    #[serde(skip)]
    pub handle: Option<SoundHandle>,
    pub id: u64,
}

impl MetaSound {
    pub fn with_path<P: AsRef<Path>>(&self, path: P) -> Result<Self> {
        //TODO: Load tags, id
        let metadata = std::fs::metadata(path.as_ref())?;
        let f = taglib::File::new(path.as_ref()).map_err(|e| anyhow!("{:?}", e))?;
        let tag = f.tag().map_err(|e| anyhow!("{:?}", e))?;
        let title = tag.title().ok_or(anyhow!("Can't read title"))?;
        
        Ok(Self {
            path: path.as_ref().into(),
            id: metadata.len(),
            name: title,
            ..self.clone()
        })
    }

    pub fn load(&self, manager: &mut AudioManager) -> Result<SoundHandle, Error> {
        manager
            .load_sound(&self.path, SoundSettings::default())
            .map_err(|e| anyhow!("{}", e))
    }

    pub fn load_mut(&mut self, manager: &mut AudioManager) -> Result<()> {
        self.handle = self.load(manager).ok();
        if let Some(handle) = &self.handle {
            self.duration = Duration::from_secs_f64(handle.duration());
        }
        Ok(())
    }

    // pub fn play(&self) -> Result<()> {
    //     let sound_handle = manager.load_sound(&self.path, SoundSettings::default())?;
    //     self.handle = Some(sound_handle);
    //     Ok(())
    // }
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
