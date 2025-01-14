use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;

/// All versions of the game represented as data that can be serialized and deserialized.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "version", content = "data")]
enum GameState {
   #[serde(rename = "1.0")]
   V1_0(V1SaveState),
   #[serde(rename = "2.0")]
   V2_0(V2SaveState),
   #[serde(rename = "3.0")]
   V3_0(V3SaveState),
}
/// Latest serializable state of the game that matches the latest version of the game.
type LatestSaveStateVersion = V3SaveState;

impl GameState {
   const DATA_FIELD_NAME: &'static str = "data";
   const VERSION_FIELD_NAME: &'static str = "version";

   fn upgrade_version(self) -> GameState {
      let v = match self {
         GameState::V1_0(x) => GameState::V2_0(x.upgrade()),
         GameState::V2_0(x) => GameState::V3_0(x.upgrade()),
         GameState::V3_0(x) => GameState::V3_0(x),
      };
      v
   }

   fn convert_to_latest(self) -> GameState {
      let mut cc = self.upgrade_version();
      // When you create a new version (latest) of the game, switch it to match here:
      while !matches!(cc, GameState::V3_0(_)) {
         cc = cc.upgrade_version();
      }
      cc
   }

   fn init_from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
      let parsed: Value = serde_json::from_str(json)?;

      // load the matching version state:
      let deserialized_version = match parsed[Self::VERSION_FIELD_NAME].as_str() {
         Some("1.0") => {
            let v: V1SaveState = serde_json::from_value(parsed[Self::DATA_FIELD_NAME].clone())?;
            GameState::V1_0(v)
         }
         Some("2.0") => {
            let v: V2SaveState = serde_json::from_value(parsed[Self::DATA_FIELD_NAME].clone())?;
            GameState::V2_0(v)
         }
         Some("3.0") => {
            let v: V3SaveState = serde_json::from_value(parsed[Self::DATA_FIELD_NAME].clone())?;
            GameState::V3_0(v)
         }
         _ => return Err("Unknown version".into()),
      };

      // convert to latest save game state version if needed;
      let latest = deserialized_version.convert_to_latest();
      Ok(latest)
   }

   fn save_to_json(&self) -> serde_json::Result<String> {
      let v = match self {
         GameState::V1_0(state) => serde_json::to_string(&GameState::V1_0(state.clone())),
         GameState::V2_0(state) => serde_json::to_string(&GameState::V2_0(state.clone())),
         GameState::V3_0(state) => serde_json::to_string(&GameState::V3_0(state.clone())),
      };
      v
   }
}

// ------------------------------------------------------
// Version 1
//
// Structures needed to convert version 1 of the game to serializable form.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1SavePlayer {
   entity: hecs::Entity,
   health: u32,
   level: u32,
   target: Option<hecs::Entity>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1SaveMonster {
   entity: hecs::Entity,
   health: u32,
}

/// V1 save state.
///
/// No changes, this is the first version.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1SaveState {
   players: Vec<V1SavePlayer>,
   monsters: Vec<V1SaveMonster>,
}

impl V1SaveState {
   fn generate_save_file() -> V1SaveState {
      struct Player {
         level: u32,
      }
      struct Health {
         health: u32,
      }
      struct Monster {}

      let mut world = hecs::World::new();

      let player = world.reserve_entity();
      world.insert(player, (Player { level: 1 }, Health { health: 20 }));

      let monster = world.reserve_entity();
      world.insert(monster, (Monster {}, Health { health: 20 }));

      let mut players = vec![];
      for (e, (a, b)) in &mut world.query::<(&Player, &Health)>() {
         players.push(V1SavePlayer {
            entity: e,
            health: b.health,
            level: a.level,
            target: Some(monster),
         })
      }

      let mut monsters = vec![];
      for (e, (a, b)) in &mut world.query::<(&Monster, &Health)>() {
         monsters.push(V1SaveMonster {
            entity: e,
            health: b.health,
         })
      }

      V1SaveState {
         players: players,
         monsters: monsters,
      }
   }

   fn upgrade(self) -> V2SaveState {
      let players = self
         .players
         .iter()
         .map(|x| V2SavePlayer {
            entity: x.entity,
            health: x.health,
            level: x.level,
            target: x.target,
            exp: 0,
            damage: 5, // we add ability for the player to attack in this version
         })
         .collect::<Vec<_>>();

      let monsters = self
         .monsters
         .iter()
         .map(|x| V2SaveMonster {
            entity: x.entity,
            health: x.health,
            damage: 2,
         })
         .collect::<Vec<_>>();

      V2SaveState {
         players: players,
         monsters: monsters,
      }
   }
}

// ------------------------------------------------------
// Version 2
//
// Structures needed to convert version 2 of the game to serializable form.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2SavePlayer {
   // v1
   entity: hecs::Entity,
   health: u32,
   level: u32,
   target: Option<hecs::Entity>,

   // v2
   exp: u32,
   damage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2SaveMonster {
   entity: hecs::Entity,
   health: u32,

   // v2
   damage: u32,
}

impl V2SaveState {
   fn upgrade(self) -> V3SaveState {
      let players = self
         .players
         .iter()
         .map(|x| V3SavePlayer {
            entity: x.entity,
            health: x.health,
            level: x.level,
            target: x.target,
            damage: x.damage,
         })
         .collect::<Vec<_>>();

      let monsters = self
         .monsters
         .iter()
         .map(|x| V3SaveMonster {
            entity: x.entity,
            health: x.health,
            damage: x.damage,
            variant: V3MonsterVariant::Angry,
         })
         .collect::<Vec<_>>();

      V3SaveState {
         players: players,
         monsters: monsters,
      }
   }
}

/// V2 save state.
///
/// In this version we added the following changes to the game:
///
/// - Add `exp` field for player.
/// - Add `damage` field for player.
/// - Add `damage` field for monster.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2SaveState {
   players: Vec<V2SavePlayer>,
   monsters: Vec<V2SaveMonster>,
}

// ------------------------------------------------------
// Version 3
//
// Structures needed to convert version 3 of the game to serializable form.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SavePlayer {
   // v1
   entity: hecs::Entity,
   health: u32,
   level: u32,
   target: Option<hecs::Entity>,

   // v2
   // exp: u32, // v3 remove
   damage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum V3MonsterVariant {
   Angry,
   Scary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SaveMonster {
   entity: hecs::Entity,
   health: u32,
   damage: u32,

   // v3
   variant: V3MonsterVariant,
}

/// V3 save state.
///
/// In this version we added the following changes to the game:
///
/// - Remove `exp` field from player.
/// - Add `MonsterVariant` enum field for monster.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SaveState {
   players: Vec<V3SavePlayer>,
   monsters: Vec<V3SaveMonster>,
}

// ------------------------------------------------------

/// Loads the latest save game state to the current version of the game.
fn run_latest_version_of_game_from_save_state(state: LatestSaveStateVersion) {
   #[derive(Debug, strum_macros::Display)]
   enum MonsterVariant {
      Angry,
      Scary,
   }

   impl From<V3MonsterVariant> for MonsterVariant {
      fn from(value: V3MonsterVariant) -> Self {
         match value {
            V3MonsterVariant::Angry => Self::Angry,
            V3MonsterVariant::Scary => Self::Scary,
         }
      }
   }

   struct Player {
      level: u32,
      damage: u32,
      target: Option<hecs::Entity>,
   }

   struct Health {
      health: u32,
   }

   struct Monster {
      damage: u32,
      variant: MonsterVariant,
   }

   let mut world = hecs::World::new();

   for x in state.players {
      let bundle = (
         Player {
            level: x.level,
            damage: x.damage,
            target: x.target,
         },
         Health { health: x.health },
      );
      world.spawn_at(x.entity, bundle);
   }

   for x in state.monsters {
      let bundle = (
         Monster {
            damage: x.damage,
            variant: x.variant.into(),
         },
         Health { health: x.health },
      );
      world.spawn_at(x.entity, bundle);
   }

   // run game:

   println!("Running game simulation from loaded save:");

   for (e, (a, b)) in &mut world.query::<(&Player, &Health)>() {
      println!(
         "Player entity {}: (Health: {}) (Damage: {}) (Level: {})",
         e.id(),
         b.health,
         a.damage,
         a.level
      );

      // player attacks target with health component (monster):
      //
      // (this tests that the entity relations have stayed consistent across
      // different save game conversions)
      if let Some(target_e) = a.target {
         let mut x = world
            .get::<(&mut Health)>(target_e)
            .expect("target should be valid");
         x.health = x.health.saturating_sub(a.damage);
      }
   }

   for (e, (a, b)) in &mut world.query::<(&Monster, &Health)>() {
      println!(
         "Monster entity {}: (Health: {}) (Damage: {}) (Variant: {})",
         e.id(),
         b.health,
         a.damage,
         a.variant
      );
   }
}

fn save_to_disk<T: Serialize>(path: &Path, data: T) -> Result<(), std::io::Error> {
   let json = serde_json::to_string(&data)?;
   let mut file = File::create(&path)?;
   file.write(&json.as_bytes())?;
   println!("Saved \'{}\' to disk.", path.display());
   Ok(())
}

fn load_from_disk_as_json_string(path: &Path) -> Result<String, std::io::Error> {
   let mut open = File::open(path)?;
   let mut json = String::new();
   open.read_to_string(&mut json);
   Ok(json)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   // load save file from disk. If save file does not exist we generate it.
   let save_file_path = Path::new("version1_save");
   if !save_file_path.exists() {
      let v1_game_state = GameState::V1_0(V1SaveState::generate_save_file());
      save_to_disk(save_file_path, &v1_game_state)?;
   }
   let json = load_from_disk_as_json_string(&save_file_path)?;

   // convert the loaded save file to latest format:
   let mut loaded_state = GameState::init_from_json(&json)?;

   // load the save file and start the game:
   match loaded_state {
      GameState::V1_0(_) => {}
      GameState::V2_0(_) => {}
      GameState::V3_0(x) => run_latest_version_of_game_from_save_state(x),
   }
   Ok(())
}
