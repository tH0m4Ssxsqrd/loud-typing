use rodio::source::Buffered;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct Sound {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    audio_sources: Vec<Buffered<Decoder<BufReader<fs::File>>>>,
    total_files: i32,
    total_keys: i32,
}

impl Sound {
    pub fn new(paths: &[PathBuf], total_keys: i32) -> Result<Self, Box<dyn Error>> {
        let (stream, handle) = OutputStream::try_default()?;

        let mut sources = Vec::new();

        for path in paths {
            if path.is_dir() {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        Self::process_file(&path, &mut sources)?;
                    }
                }
            } else if path.is_file() {
                Self::process_file(path, &mut sources)?;
            }
        }
        let total_files = i32::try_from(paths.len()).expect("Could not count files.");

        Ok(Self {
            _stream: stream,
            handle,
            audio_sources: sources,
            total_files,
            total_keys,
        })
    }

    pub fn play(&self, key: i32) -> Result<(), Box<dyn Error>> {
        let key = key.max(1);

        let base = self.total_keys;
        let modulus = self.total_files;
        let exponent = key;

        let index = modular_pow(base, modulus, exponent);

        // Wrap the index around if it's larger than the length of audio_sources
        #[allow(clippy::cast_sign_loss)]
        let index = (index as usize) % self.audio_sources.len();

        let source = &self.audio_sources[index];
        self.handle.play_raw(source.clone().convert_samples())?;

        Ok(())
    }

    fn process_file(path: &Path, sources: &mut Vec<Buffered<Decoder<BufReader<File>>>>) -> Result<(), Box<dyn Error>> {
        let file = BufReader::new(File::open(path)?);
        if let Ok(decoder) = Decoder::new(file) {
            if decoder.total_duration().unwrap_or_default() <= std::time::Duration::from_secs(5) {
                sources.push(decoder.buffered());
            }
        }
        Ok(())
    }
}

fn modular_pow(base: i32, exponent: i32, modulus: i32) -> i32 {
    if modulus == 1 {
        return 0;
    }

    let mut result = 1;
    let base_mod = base % modulus;

    (0..exponent).for_each(|_| {
        result = (result * base_mod) % modulus;
    });

    result
}
