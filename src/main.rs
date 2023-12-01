mod player;

use crate::player::Sound;
use clap::Parser;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "104")]
    total_keys: i32,
}

const fn key_up(_key: Keycode) {}

fn key_down(key_number: i32, player: &Sound, _total_keys: i32) -> Result<(), Box<dyn Error>> {
    player.play(key_number)?;

    Ok(())
}

fn get_file_paths(dir: &str) -> Vec<PathBuf> {
    let mut file_paths = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            // If the entry is a file, add its path to the vector
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    file_paths.push(entry.path());
                }
            }
        }
    }

    file_paths
}

#[tokio::main]
async fn main() {
  let args = Args::parse();
  let total_keys = args.total_keys;

  let device_state = DeviceState::new();
  let mut key_states: HashMap<Keycode, bool> = HashMap::new();

  let paths = get_file_paths("./sounds");

  let player = Arc::new(tokio::sync::Mutex::new(Sound::new(&paths, total_keys).expect("Couldn't create sound player.")));

  let (tx, mut rx) = tokio::sync::mpsc::channel::<Keycode>(100);

  let player_clone = Arc::clone(&player);
  let _handle = tokio::spawn(async move {
      while let Some(key) = rx.recv().await {
          let key_as_i32 = key as i32;
          let player = player_clone.lock().await;
          key_down(key_as_i32, &*player, total_keys)
              .map_err(|err| eprintln!("ERROR: {err}"))
              .expect("Couldn't process event.");
      }
  });

  loop {
      let keys: Vec<Keycode> = device_state.get_keys();

      // Check for key down events
      for key in &keys {
          if key_states.get(key).is_none() {
              tx.send(*key).await.unwrap();
              key_states.insert(*key, true);
          }
      }

      // Check for key up events
      let keys_up: Vec<Keycode> = key_states.keys().copied().collect();
      for key in keys_up {
          if !keys.contains(&key) {
              key_up(key);
              key_states.remove(&key);
          }
      }
  }
}