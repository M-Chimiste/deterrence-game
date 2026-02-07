use crate::state::campaign_state::CampaignState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Full save data written to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub campaign: CampaignState,
    pub wave_number: u32,
    pub seed: u64,
    pub timestamp: u64,
    pub slot_name: String,
}

/// Lightweight metadata for listing saves without loading full state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub slot_name: String,
    pub wave_number: u32,
    pub timestamp: u64,
    pub resources: u32,
}

fn save_path(dir: &Path, slot: &str) -> std::path::PathBuf {
    dir.join(format!("{}.json", slot))
}

pub fn save_to_file(dir: &Path, slot: &str, data: &SaveData) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create save directory: {e}"))?;
    let path = save_path(dir, slot);
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize save data: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write save file: {e}"))?;
    Ok(())
}

pub fn load_from_file(dir: &Path, slot: &str) -> Result<SaveData, String> {
    let path = save_path(dir, slot);
    let json = fs::read_to_string(&path).map_err(|e| format!("Failed to read save file: {e}"))?;
    let data: SaveData =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse save data: {e}"))?;
    Ok(data)
}

pub fn list_saves(dir: &Path) -> Vec<SaveMetadata> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut saves = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Ok(json) = fs::read_to_string(&path)
            && let Ok(data) = serde_json::from_str::<SaveData>(&json)
        {
            saves.push(SaveMetadata {
                slot_name: data.slot_name,
                wave_number: data.wave_number,
                timestamp: data.timestamp,
                resources: data.campaign.resources,
            });
        }
    }
    saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    saves
}

pub fn delete_save(dir: &Path, slot: &str) -> Result<(), String> {
    let path = save_path(dir, slot);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete save file: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_save_data(slot: &str, wave: u32) -> SaveData {
        SaveData {
            campaign: CampaignState::default(),
            wave_number: wave,
            seed: 42,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            slot_name: slot.to_string(),
        }
    }

    #[test]
    fn save_data_roundtrip() {
        let data = make_save_data("test", 5);
        let json = serde_json::to_string(&data).unwrap();
        let restored: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.wave_number, 5);
        assert_eq!(restored.seed, 42);
        assert_eq!(restored.slot_name, "test");
        assert_eq!(restored.campaign.resources, data.campaign.resources);
        assert_eq!(
            restored.campaign.owned_regions.len(),
            data.campaign.owned_regions.len()
        );
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join("deterrence_test_save_load");
        let _ = fs::remove_dir_all(&dir);

        let data = make_save_data("slot1", 3);
        save_to_file(&dir, "slot1", &data).unwrap();
        let loaded = load_from_file(&dir, "slot1").unwrap();
        assert_eq!(loaded.wave_number, 3);
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.campaign.resources, data.campaign.resources);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_saves_empty() {
        let dir = std::env::temp_dir().join("deterrence_test_list_empty");
        let _ = fs::remove_dir_all(&dir);
        let saves = list_saves(&dir);
        assert!(saves.is_empty());
    }

    #[test]
    fn list_saves_multiple() {
        let dir = std::env::temp_dir().join("deterrence_test_list_multi");
        let _ = fs::remove_dir_all(&dir);

        let mut data1 = make_save_data("early", 2);
        data1.timestamp = 1000;
        save_to_file(&dir, "early", &data1).unwrap();

        let mut data2 = make_save_data("late", 8);
        data2.timestamp = 2000;
        save_to_file(&dir, "late", &data2).unwrap();

        let saves = list_saves(&dir);
        assert_eq!(saves.len(), 2);
        // Sorted by timestamp descending
        assert_eq!(saves[0].slot_name, "late");
        assert_eq!(saves[1].slot_name, "early");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_save_removes_file() {
        let dir = std::env::temp_dir().join("deterrence_test_delete");
        let _ = fs::remove_dir_all(&dir);

        let data = make_save_data("todelete", 1);
        save_to_file(&dir, "todelete", &data).unwrap();
        assert!(save_path(&dir, "todelete").exists());

        delete_save(&dir, "todelete").unwrap();
        assert!(!save_path(&dir, "todelete").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_nonexistent_save_ok() {
        let dir = std::env::temp_dir().join("deterrence_test_delete_noop");
        delete_save(&dir, "nope").unwrap();
    }

    #[test]
    fn simulation_to_save_data() {
        use crate::engine::simulation::Simulation;

        let mut sim = Simulation::new_with_seed(99);
        sim.setup_world();
        sim.wave_number = 5;

        let data = sim.to_save_data("manual");
        assert_eq!(data.slot_name, "manual");
        assert_eq!(data.wave_number, 5);
        assert_eq!(data.seed, 99);
        assert_eq!(data.campaign.resources, sim.campaign.resources);
        assert!(data.timestamp > 0);
    }

    #[test]
    fn simulation_from_save_data() {
        use crate::engine::simulation::Simulation;
        use crate::state::game_state::GamePhase;

        let mut sim = Simulation::new_with_seed(77);
        sim.setup_world();
        sim.wave_number = 10;
        sim.campaign.resources = 250;

        let data = sim.to_save_data("restore");
        let restored = Simulation::from_save_data(data);

        assert_eq!(restored.wave_number, 10);
        assert_eq!(restored.seed, 77);
        assert_eq!(restored.campaign.resources, 250);
        assert_eq!(restored.phase, GamePhase::Strategic);
        // World should be rebuilt with cities and batteries
        assert!(!restored.city_ids.is_empty());
        assert!(!restored.battery_ids.is_empty());
    }
}
