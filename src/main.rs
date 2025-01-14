use serde::{Deserialize, Serialize};
use serde_json::Value;

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

impl GameState {
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
        while !matches!(cc, GameState::V3_0(_)) {
            cc = cc.upgrade_version();
        }
        cc
    }

    fn load_from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: Value = serde_json::from_str(json)?;
        let deserialized_version = match parsed["version"].as_str() {
            Some("1.0") => {
                let v: V1SaveState = serde_json::from_value(parsed["data"].clone())?;
                GameState::V1_0(v)
            }
            Some("2.0") => {
                let v: V2SaveState = serde_json::from_value(parsed["data"].clone())?;
                GameState::V2_0(v)
            }
            Some("3.0") => {
                let v: V3SaveState = serde_json::from_value(parsed["data"].clone())?;
                GameState::V3_0(v)
            }
            _ => return Err("Unknown version".into()),
        };
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1SavePlayer {
    entity: hecs::Entity,
    health: u32,
    level: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V1SaveMonster {
    entity: hecs::Entity,
    health: u32,
}

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
                health: a.level,
                level: b.health,
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
                exp: 0,
                damage: 0,
            })
            .collect::<Vec<_>>();

        let monsters = self
            .monsters
            .iter()
            .map(|x| V2SaveMonster {
                entity: x.entity,
                health: x.health,
                damage: 5,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2SavePlayer {
    // v1
    entity: hecs::Entity,
    health: u32,
    level: u32,

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
                variant: MonsterVariant::Angry,
            })
            .collect::<Vec<_>>();

        V3SaveState {
            players: players,
            monsters: monsters,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V2SaveState {
    players: Vec<V2SavePlayer>,
    monsters: Vec<V2SaveMonster>,
}

// ------------------------------------------------------
// Version 3

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SavePlayer {
    // v1
    entity: hecs::Entity,
    health: u32,
    level: u32,

    // v2
    // exp: u32, // v3 remove
    damage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MonsterVariant {
    Angry,
    Scary,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SaveMonster {
    entity: hecs::Entity,
    health: u32,

    // v2
    damage: u32,

    // v3
    variant: MonsterVariant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct V3SaveState {
    players: Vec<V3SavePlayer>,
    monsters: Vec<V3SaveMonster>,
}

// ------------------------------------------------------

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let v1_game_state = GameState::V1_0(V1SaveState::generate_save_file());
    let json_v1 = v1_game_state.save_to_json()?;
    let mut loaded_state = GameState::load_from_json(&json_v1)?;
    println!("Loaded and converted game state: {:?}", loaded_state);
    Ok(())
}
