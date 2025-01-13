use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "version", content = "data")]
enum GameState {
   #[serde(rename = "1.0")]
   V1_0(V1_0State),
   #[serde(rename = "2.0")]
   V2_0(V2_0State),
   #[serde(rename = "3.0")]
   V3_0(V3_0State),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1_0State {
   health: u32,
   level: u32,
}


impl V1_0State {
   fn convert_to_v2_0(self) -> V2_0State {
      V2_0State {
         health: self.health,
         level: self.level,
         mana: 0,
      }
   }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2_0State {
   health: u32,
   level: u32,
   mana: u32, // New field in version 2.0
}

impl V2_0State {
   fn convert_to_v3_0(self) -> V3_0State {
      V3_0State {
         health: self.health,
         level: self.level,
         mana: self.mana,
         exp: 0,
      }
   }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3_0State {
   health: u32,
   level: u32,
   mana: u32, // New field in version 2.0
   exp: u32,
}

impl GameState {
   fn upgrade_version(self) -> GameState {
      let v = match self {
         GameState::V1_0(x) => {
            GameState::V2_0(x.convert_to_v2_0())
         }
         GameState::V2_0(x) => {
            GameState::V3_0(x.clone().convert_to_v3_0())
         }
         GameState::V3_0(x) => {
            GameState::V3_0(x)
         }
      };
      v
   }

   fn convert_to_latest(self) -> GameState {
      let mut cc = self.upgrade_version();
      while !matches!(cc, GameState::V3_0(_)) {
         cc = cc.upgrade_version();
      }
      cc
   }

   // Method to load from JSON string, handling version conversion
   fn load_from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
      let parsed: Value = serde_json::from_str(json)?;
      let deserialized_version = match parsed["version"].as_str() {
         Some("1.0") => {
            let v: V1_0State = serde_json::from_value(parsed["data"].clone())?;
            GameState::V1_0(v)
         },
         Some("2.0") => {
            let v: V2_0State = serde_json::from_value(parsed["data"].clone())?;
            GameState::V2_0(v)
         },
         Some("3.0") => {
            let v: V3_0State = serde_json::from_value(parsed["data"].clone())?;
            GameState::V3_0(v)
         },
         _ => return Err("Unknown version".into()),
      };
      let latest = deserialized_version.convert_to_latest();
      Ok(latest)
   }

   // Serialize the current state (always to the latest version)
   fn save_to_json(&self) -> serde_json::Result<String> {
      let v = match self {
         GameState::V1_0(state) => serde_json::to_string(&GameState::V1_0(state.clone())),
         GameState::V2_0(state) => serde_json::to_string(&GameState::V2_0(state.clone())),
         GameState::V3_0(state) => serde_json::to_string(&GameState::V3_0(state.clone()))
      };
      v
   }
}


struct Player {
   level: u32,
   mana: u32,
   exp: u32,
}

struct Health {
   health: u32,
}

struct Monster {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveV1Player {
   entity: hecs::Entity,
   health: u32,
   level: u32,
   mana: u32,
   exp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveV1Monster {
   entity: hecs::Entity,
   health: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SaveV1 {
   players: Vec<SaveV1Player>,
   monsters: Vec<SaveV1Monster>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   // Example of saving a V1_0 game state
   let v1_game_state = GameState::V1_0(V1_0State {
      health: 100,
      level: 1,
   });
   let json_v1 = v1_game_state.save_to_json()?;

   // Now we "update" the game to version 2.0, but we load a V1.0 save
   let mut loaded_state = GameState::load_from_json(&json_v1)?;

   // Print or use the loaded state, which is now in the V2_0 format
   println!("Loaded and converted game state: {:?}", loaded_state);


   let mut world = hecs::World::new();

   let player =

   world.spawn((Player ,))


   Ok(())
}
