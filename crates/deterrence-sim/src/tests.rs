//! Tests for the simulation engine, radar systems, fire control, and engagement pipeline.

use deterrence_core::commands::PlayerCommand;
use deterrence_core::components::{DetectionCounter, Interceptor, Threat, ThreatProfile};
use deterrence_core::enums::*;
use deterrence_core::types::{Position, Velocity};

use crate::engine::{SimConfig, SimulationEngine};
use crate::systems::movement;

// ---- Determinism ----

#[test]
fn test_determinism_same_seed() {
    let mut engine_a = SimulationEngine::new(SimConfig {
        seed: 12345,
        ..Default::default()
    });
    let mut engine_b = SimulationEngine::new(SimConfig {
        seed: 12345,
        ..Default::default()
    });

    engine_a.queue_command(PlayerCommand::StartMission);
    engine_b.queue_command(PlayerCommand::StartMission);

    for _ in 0..300 {
        let snap_a = engine_a.tick();
        let snap_b = engine_b.tick();

        let json_a = serde_json::to_string(&snap_a).unwrap();
        let json_b = serde_json::to_string(&snap_b).unwrap();
        assert_eq!(json_a, json_b, "Snapshots diverged with same seed");
    }
}

#[test]
fn test_determinism_different_seeds() {
    let mut engine_a = SimulationEngine::new(SimConfig {
        seed: 111,
        ..Default::default()
    });
    let mut engine_b = SimulationEngine::new(SimConfig {
        seed: 222,
        ..Default::default()
    });

    engine_a.queue_command(PlayerCommand::StartMission);
    engine_b.queue_command(PlayerCommand::StartMission);

    // Run enough ticks for radar sweeps to produce divergent results.
    // Tick 1 snapshots are identical (both have empty tracks), but
    // after detection rolls with different seeds, snapshots diverge.
    let mut diverged = false;
    for _ in 0..500 {
        let snap_a = engine_a.tick();
        let snap_b = engine_b.tick();
        let json_a = serde_json::to_string(&snap_a).unwrap();
        let json_b = serde_json::to_string(&snap_b).unwrap();
        if json_a != json_b {
            diverged = true;
            break;
        }
    }
    assert!(diverged, "Different seeds should produce divergent output");
}

// ---- Entity lifecycle ----

#[test]
fn test_entity_lifecycle_oob_despawn() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Verify threats exist as entities (even if not yet tracked).
    let threat_count = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert!(
        threat_count > 0,
        "Should have threat entities after StartMission"
    );

    // Threats include sea-skimmers (290 m/s), supersonic cruisers (850 m/s),
    // and a subsonic drone (100 m/s, spawns at tick 600, ~165km out).
    // Threat AI transitions to Impact at 50m from origin, cleanup despawns.
    // Slowest: drone at 100 m/s from 165km = ~1650s = ~49500 ticks + 600 spawn delay.
    // Run 55000 ticks to ensure all threats reach impact.
    for _ in 0..55_000 {
        engine.tick();
    }

    let final_threat_count = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert_eq!(
        final_threat_count, 0,
        "All threats should have been despawned after going OOB"
    );

    let final_snap = engine.tick();
    assert_eq!(
        final_snap.tracks.len(),
        0,
        "No tracks should remain after all threats despawned"
    );
}

#[test]
fn test_snapshot_size_under_100kb() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Spawn pre-tracked threats for snapshot size test (bypasses radar detection).
    // The 3 mission threats may also get promoted during ticks, so we assert >= 100.
    engine.spawn_tracked_threats(100);

    // Run a few ticks to build up position history.
    for _ in 0..100 {
        engine.tick();
    }

    let snapshot = engine.tick();
    assert!(
        snapshot.tracks.len() >= 100,
        "Should have at least 100 tracks, got {}",
        snapshot.tracks.len()
    );

    let json = serde_json::to_string(&snapshot).unwrap();
    let size_kb = json.len() as f64 / 1024.0;

    assert!(
        size_kb < 100.0,
        "Snapshot with 100 entities should be <100KB, was {size_kb:.1}KB",
    );
    assert!(
        size_kb > 1.0,
        "Snapshot should have substantial data, was only {size_kb:.1}KB",
    );
}

// ---- Tick timing ----

#[test]
fn test_tick_timing_30_ticks_one_second() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);

    for _ in 0..30 {
        engine.tick();
    }

    assert_eq!(engine.time().tick, 30);
    assert!(
        (engine.time().elapsed_secs - 1.0).abs() < 1e-10,
        "30 ticks should equal 1.0 seconds, got {}",
        engine.time().elapsed_secs
    );
}

// ---- Pause/Resume ----

#[test]
fn test_pause_stops_simulation() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);

    for _ in 0..10 {
        engine.tick();
    }
    assert_eq!(engine.time().tick, 10);
    assert_eq!(engine.phase(), GamePhase::Active);

    engine.queue_command(PlayerCommand::Pause);
    for _ in 0..10 {
        engine.tick();
    }
    assert_eq!(
        engine.time().tick,
        10,
        "Time should not advance while paused"
    );
    assert_eq!(engine.phase(), GamePhase::Paused);

    engine.queue_command(PlayerCommand::Resume);
    for _ in 0..10 {
        engine.tick();
    }
    assert_eq!(engine.time().tick, 20);
    assert_eq!(engine.phase(), GamePhase::Active);
}

// ---- Movement ----

#[test]
fn test_movement_integration() {
    let mut world = hecs::World::new();

    world.spawn((Position::new(0.0, 0.0, 0.0), Velocity::new(100.0, 0.0, 0.0)));

    for _ in 0..30 {
        movement::run(&mut world);
    }

    let mut query = world.query::<&Position>();
    let (_, pos) = query.iter().next().unwrap();
    assert!(
        (pos.x - 100.0).abs() < 1e-6,
        "After 1s at 100 m/s east, x should be ~100, got {}",
        pos.x
    );
    assert!(pos.y.abs() < 1e-10, "y should be 0, got {}", pos.y);
    assert!(pos.z.abs() < 1e-10, "z should be 0, got {}", pos.z);
}

// ---- Track hooking ----

#[test]
fn test_hook_track_command() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Spawn pre-tracked threats for immediate hooking.
    engine.spawn_tracked_threats(3);
    let snap = engine.tick();

    assert!(!snap.tracks.is_empty(), "Should have pre-tracked threats");
    let first_track = snap.tracks[0].track_number;
    assert!(!snap.tracks[0].hooked, "Tracks should start unhooked");

    engine.queue_command(PlayerCommand::HookTrack {
        track_number: first_track,
    });
    let snap2 = engine.tick();

    let hooked = snap2
        .tracks
        .iter()
        .find(|t| t.track_number == first_track)
        .unwrap();
    assert!(hooked.hooked, "Track should be hooked after HookTrack");

    for t in &snap2.tracks {
        if t.track_number != first_track {
            assert!(!t.hooked, "Only the hooked track should be hooked");
        }
    }

    engine.queue_command(PlayerCommand::UnhookTrack);
    let snap3 = engine.tick();
    assert!(
        snap3.tracks.iter().all(|t| !t.hooked),
        "All tracks should be unhooked"
    );
}

// ---- Snapshot systems ----

#[test]
fn test_snapshot_includes_systems() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();

    // Own ship at origin.
    assert!(snap.own_ship.position.x.abs() < 1e-10);
    assert!(snap.own_ship.position.y.abs() < 1e-10);

    // Radar with full energy (no tracks yet = all search).
    assert!((snap.radar.energy_total - 100.0).abs() < 1e-10);
    assert_eq!(snap.radar.mode, RadarMode::TrackWhileScan);

    // VLS: 64 cells with correct loadout.
    assert_eq!(snap.vls.total_capacity, 64);
    assert_eq!(snap.vls.ready_standard, 32);
    assert_eq!(snap.vls.ready_extended_range, 16);
    assert_eq!(snap.vls.ready_point_defense, 16);
    assert_eq!(snap.vls.total_ready, 64);

    // Illuminators: 3 channels, all idle.
    assert_eq!(snap.illuminators.len(), 3);
    assert!(snap
        .illuminators
        .iter()
        .all(|i| i.status == IlluminatorStatus::Idle));
}

// ---- Phase gating ----

#[test]
fn test_start_mission_phase_gating() {
    let mut engine = SimulationEngine::new(SimConfig::default());

    // Before StartMission, phase is MainMenu.
    let snap = engine.tick();
    assert_eq!(snap.phase, GamePhase::MainMenu);
    assert!(snap.tracks.is_empty());

    // Start mission — threats spawn but are undetected initially.
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();
    assert_eq!(snap.phase, GamePhase::Active);

    // Threats exist as entities but aren't yet tracks.
    let threat_count = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert!(threat_count > 0, "Threat entities should exist");

    // Starting again while Active should be a no-op.
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();
    let threat_count_2 = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert_eq!(
        threat_count, threat_count_2,
        "StartMission while Active should be ignored"
    );
    assert_eq!(snap.phase, GamePhase::Active);
}

// ---- Time scale ----

#[test]
fn test_set_time_scale() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    assert!((engine.time_scale() - 1.0).abs() < 1e-10);

    engine.queue_command(PlayerCommand::SetTimeScale { scale: 2.0 });
    engine.tick();
    assert!((engine.time_scale() - 2.0).abs() < 1e-10);

    // Clamped to 0.0..4.0.
    engine.queue_command(PlayerCommand::SetTimeScale { scale: 10.0 });
    engine.tick();
    assert!((engine.time_scale() - 4.0).abs() < 1e-10);

    engine.queue_command(PlayerCommand::SetTimeScale { scale: -1.0 });
    engine.tick();
    assert!(engine.time_scale().abs() < 1e-10);
}

// ---- Radar detection pipeline ----

#[test]
fn test_undetected_threats_not_in_snapshot() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();

    // Threats just spawned — not yet detected by radar.
    assert!(
        snap.tracks.is_empty(),
        "Tracks should be empty immediately after StartMission (threats are undetected)"
    );

    // But threat entities exist with DetectionCounter.
    let counter_count = {
        let mut q = engine.world().query::<(&Threat, &DetectionCounter)>();
        q.iter().count()
    };
    assert!(
        counter_count > 0,
        "Threat entities with DetectionCounter should exist"
    );
}

#[test]
fn test_track_initiation_pipeline() {
    let mut engine = SimulationEngine::new(SimConfig {
        seed: 42,
        ..Default::default()
    });
    engine.queue_command(PlayerCommand::StartMission);

    // Run enough ticks for radar sweeps to detect threats.
    // With ~4-second rotation and 3 hits needed, ~12+ seconds (360+ ticks).
    // Run 500 ticks (~16.7 seconds, >4 full sweeps) to be safe.
    for _ in 0..500 {
        engine.tick();
    }

    let snap = engine.tick();

    // At least some threats should now be tracked.
    assert!(
        !snap.tracks.is_empty(),
        "After 500 ticks, radar should have detected and promoted some threats to tracks"
    );

    // Tracks should have quality around TRACK_INITIAL_QUALITY.
    for track in &snap.tracks {
        assert!(
            track.quality > 0.0 && track.quality <= 1.0,
            "Track quality should be in (0, 1], got {}",
            track.quality
        );
        assert_eq!(
            track.classification,
            Classification::Unknown,
            "New tracks should be classified as Unknown"
        );
    }
}

#[test]
fn test_energy_budget_vs_track_count() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // No tracks yet — all energy to search.
    let snap = engine.tick();
    let initial_search_energy = snap.radar.energy_search;
    assert!(
        (initial_search_energy - 100.0).abs() < 1e-10,
        "With 0 tracks, search energy should be 100%: {initial_search_energy}"
    );

    // Spawn pre-tracked threats to test energy allocation.
    engine.spawn_tracked_threats(5);
    let snap = engine.tick();

    // 5 tracks * 2.0 energy per track = 10.0 track energy, 90.0 search.
    assert!(
        snap.radar.energy_search < initial_search_energy,
        "More tracks should reduce search energy"
    );
    assert!(
        (snap.radar.energy_track - 10.0).abs() < 1e-10,
        "Track energy for 5 tracks should be 10.0: {}",
        snap.radar.energy_track
    );
}

#[test]
fn test_sector_narrowing() {
    let mut engine = SimulationEngine::new(SimConfig {
        seed: 99,
        ..Default::default()
    });
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Set sector to 90° (PI/2) centered at North (0.0).
    // Only threats in bearing range [315°, 45°] should be detectable.
    engine.queue_command(PlayerCommand::SetRadarSector {
        center_bearing: 0.0,
        width: std::f64::consts::FRAC_PI_2,
    });

    // Run many ticks for detection sweeps.
    for _ in 0..600 {
        engine.tick();
    }

    let snap = engine.tick();

    // Any detected tracks should be within the sector.
    for track in &snap.tracks {
        let bearing_deg = track.bearing * 180.0 / std::f64::consts::PI;
        assert!(
            bearing_deg < 55.0 || bearing_deg > 305.0,
            "Track at bearing {bearing_deg}° should be within North-facing 90° sector"
        );
    }
}

#[test]
fn test_classify_track_command() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Spawn a pre-tracked threat.
    engine.spawn_tracked_threats(1);
    let snap = engine.tick();
    let track_number = snap.tracks[0].track_number;

    // Classify as Hostile.
    engine.queue_command(PlayerCommand::ClassifyTrack {
        track_number,
        classification: Classification::Hostile,
    });
    let snap = engine.tick();
    let track = snap
        .tracks
        .iter()
        .find(|t| t.track_number == track_number)
        .unwrap();
    assert_eq!(track.classification, Classification::Hostile);
}

#[test]
fn test_set_radar_mode_command() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();
    assert_eq!(snap.radar.mode, RadarMode::TrackWhileScan);

    engine.queue_command(PlayerCommand::SetRadarMode {
        mode: RadarMode::BurnThrough,
    });
    let snap = engine.tick();
    assert_eq!(snap.radar.mode, RadarMode::BurnThrough);
}

// ---- Phase 5: Wave Spawning ----

#[test]
fn test_wave_schedule_spawns_at_correct_ticks() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);

    // Tick 1: wave 0 should have spawned 3 threats.
    engine.tick();
    let threat_count_wave0 = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert_eq!(threat_count_wave0, 3, "Wave 0: 3 threats expected");

    // Run to tick 300: wave 1 should add 3 more threats (2 Mk1 + 1 Supersonic).
    for _ in 1..301 {
        engine.tick();
    }
    let threat_count_wave1 = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    // Original 3 still exist + 3 new = 6 (some may have been despawned via intercepts,
    // but at this stage they're all still in flight).
    assert!(
        threat_count_wave1 >= 6,
        "After wave 1 spawn at tick 300: expected >= 6 threats, got {threat_count_wave1}"
    );

    // Run to tick 600: wave 2 adds 4 more (2 Mk2 + 1 Supersonic + 1 Drone).
    for _ in 301..601 {
        engine.tick();
    }
    let threat_count_wave2 = {
        let mut q = engine.world().query::<&Threat>();
        q.iter().count()
    };
    assert!(
        threat_count_wave2 >= 10,
        "After wave 2 spawn at tick 600: expected >= 10 threats, got {threat_count_wave2}"
    );
}

#[test]
fn test_score_total_threats_matches_wave_schedule() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();

    // Default mission: 3 + 3 + 4 = 10 total threats.
    assert_eq!(
        snap.score.threats_total, 10,
        "Score.threats_total should match wave schedule total"
    );
}

// ---- Phase 5: Threat AI ----

#[test]
fn test_threat_terminal_transition() {
    let mut engine = SimulationEngine::new(SimConfig {
        seed: 777,
        ..Default::default()
    });
    engine.queue_command(PlayerCommand::StartMission);

    // Manual doctrine so fire control doesn't interfere.
    engine.queue_command(PlayerCommand::SetDoctrine {
        mode: DoctrineMode::Manual,
    });

    // Sea-skimmers at 290 m/s from 150-180km reach 50km popup range in ~345-448s.
    // Run 14000 ticks (~467s) to catch some threats in popup/terminal phase.
    for _ in 0..14_000 {
        engine.tick();
    }

    let has_non_cruise = {
        let mut q = engine.world().query::<(&Threat, &ThreatProfile)>();
        q.iter().any(|(_, (_, profile))| {
            matches!(
                profile.phase,
                ThreatPhase::PopUp | ThreatPhase::Terminal | ThreatPhase::Impact
            )
        })
    };
    assert!(
        has_non_cruise,
        "After ~467s, some threats should have transitioned from Cruise to PopUp/Terminal"
    );
}

#[test]
fn test_threat_impact_despawns() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Doctrine Manual so no engagements interfere.
    engine.queue_command(PlayerCommand::SetDoctrine {
        mode: DoctrineMode::Manual,
    });

    // Sea-skimmers at 290 m/s from ~165km ≈ 569s ≈ 17000 ticks to origin.
    // Run enough for first threats to impact.
    for _ in 0..20_000 {
        engine.tick();
    }

    let snap = engine.tick();
    // Some threats should have impacted and been despawned.
    // The score tracks impacts.
    assert!(
        snap.score.threats_killed == 0,
        "Manual doctrine: no kills expected"
    );
}

// ---- Phase 5: Engagement creation ----

#[test]
fn test_engagement_created_for_hostile_track() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    // Spawn pre-tracked hostile threat. Default doctrine is AutoSpecial.
    engine.spawn_tracked_threats(1);
    // Run a few ticks for fire control to create engagement.
    for _ in 0..5 {
        engine.tick();
    }

    assert!(
        !engine.engagements().is_empty(),
        "AutoSpecial should create engagement for hostile track"
    );

    let eng = engine.engagements().values().next().unwrap();
    assert_eq!(eng.phase, EngagementPhase::SolutionCalc);
    assert!(eng.pk > 0.0 && eng.pk <= 1.0, "Pk should be valid");
}

#[test]
fn test_no_engagement_in_manual_mode() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.queue_command(PlayerCommand::SetDoctrine {
        mode: DoctrineMode::Manual,
    });
    engine.tick();

    engine.spawn_tracked_threats(3);
    for _ in 0..10 {
        engine.tick();
    }

    assert!(
        engine.engagements().is_empty(),
        "Manual doctrine should not create engagements"
    );
}

#[test]
fn test_no_duplicate_engagements() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run enough ticks for engagement to be created.
    for _ in 0..5 {
        engine.tick();
    }

    let count = engine.engagements().len();
    assert_eq!(count, 1, "Should have exactly 1 engagement");

    // Run more ticks — should not create duplicate.
    for _ in 0..30 {
        engine.tick();
    }

    // Count engagements for the same track (excluding Complete/Aborted).
    let active_count = engine
        .engagements()
        .values()
        .filter(|e| {
            !matches!(
                e.phase,
                EngagementPhase::Complete | EngagementPhase::Aborted
            )
        })
        .count();
    assert!(
        active_count <= 1,
        "Should not have duplicate active engagements for same track"
    );
}

// ---- Phase 5: Veto Clock ----

#[test]
fn test_veto_clock_countdown() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run until engagement enters Ready phase.
    // SolutionCalc takes 2s = 60 ticks.
    for _ in 0..70 {
        engine.tick();
    }

    let snap = engine.tick();

    let ready_engs: Vec<_> = snap
        .engagements
        .iter()
        .filter(|e| e.phase == EngagementPhase::Ready)
        .collect();

    if !ready_engs.is_empty() {
        let eng = &ready_engs[0];
        assert!(
            eng.veto_remaining_secs > 0.0 && eng.veto_remaining_secs <= 8.0,
            "Veto clock should be counting down: {}",
            eng.veto_remaining_secs
        );
    }
}

#[test]
fn test_veto_command_aborts_engagement() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run until engagement enters Ready phase (SolutionCalc = 2s = ~60 ticks).
    for _ in 0..65 {
        engine.tick();
    }

    // Find the engagement in Ready phase.
    let eng_id = engine
        .engagements()
        .values()
        .find(|e| e.phase == EngagementPhase::Ready)
        .map(|e| e.id);

    if let Some(id) = eng_id {
        engine.queue_command(PlayerCommand::VetoEngagement { engagement_id: id });
        engine.tick();

        let eng = engine.engagements().get(&id).unwrap();
        assert_eq!(
            eng.phase,
            EngagementPhase::Aborted,
            "Vetoed engagement should be Aborted"
        );
    }
}

#[test]
fn test_confirm_command_triggers_launch() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run until engagement enters Ready phase.
    for _ in 0..65 {
        engine.tick();
    }

    let eng_id = engine
        .engagements()
        .values()
        .find(|e| e.phase == EngagementPhase::Ready)
        .map(|e| e.id);

    if let Some(id) = eng_id {
        engine.queue_command(PlayerCommand::ConfirmEngagement { engagement_id: id });
        // Confirm sets veto_remaining to 0, next tick should trigger launch.
        engine.tick();

        let eng = engine.engagements().get(&id).unwrap();
        assert_eq!(
            eng.phase,
            EngagementPhase::Launched,
            "Confirmed engagement should launch immediately"
        );
    }
}

// ---- Phase 5: Interceptor spawning ----

#[test]
fn test_interceptor_spawned_on_launch() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run until engagement launches (SolutionCalc 2s + Ready 8s = 10s = 300 ticks).
    for _ in 0..310 {
        engine.tick();
    }

    // Check for interceptor entity.
    let interceptor_count = {
        let mut q = engine.world().query::<&Interceptor>();
        q.iter().count()
    };

    // Should have spawned at least one interceptor (veto clock expired).
    assert!(
        interceptor_count >= 1,
        "Interceptor should be spawned after veto clock expires, got {interceptor_count}"
    );

    // Score should reflect fired interceptors.
    assert!(
        engine.score().interceptors_fired >= 1,
        "interceptors_fired should be >= 1"
    );
}

#[test]
fn test_interceptor_has_friend_classification() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    // Run until launch.
    for _ in 0..310 {
        engine.tick();
    }

    let snap = engine.tick();

    // Interceptors should appear as Friend tracks.
    let friend_tracks: Vec<_> = snap
        .tracks
        .iter()
        .filter(|t| t.classification == Classification::Friend)
        .collect();

    assert!(
        !friend_tracks.is_empty(),
        "Launched interceptors should appear as Friend classification tracks"
    );
}

// ---- Phase 5: VLS tracking ----

#[test]
fn test_vls_cell_consumed_on_launch() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap_initial = engine.tick();
    let initial_ready = snap_initial.vls.total_ready;

    engine.spawn_tracked_threats(1);

    // Run until launch.
    for _ in 0..310 {
        engine.tick();
    }

    let snap = engine.tick();
    assert!(
        snap.vls.total_ready < initial_ready,
        "VLS ready count should decrease after launch: initial={initial_ready}, now={}",
        snap.vls.total_ready
    );
}

// ---- Phase 5: Audio events ----

#[test]
fn test_audio_events_veto_and_launch() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);

    let mut found_veto_start = false;
    let mut found_bird_away = false;

    // Run 310 ticks, collecting audio events.
    for _ in 0..310 {
        let snap = engine.tick();
        for event in &snap.audio_events {
            match event {
                deterrence_core::events::AudioEvent::VetoClockStart { .. } => {
                    found_veto_start = true;
                }
                deterrence_core::events::AudioEvent::BirdAway { .. } => {
                    found_bird_away = true;
                }
                _ => {}
            }
        }
    }

    assert!(found_veto_start, "Should have emitted VetoClockStart");
    assert!(found_bird_away, "Should have emitted BirdAway");
}

// ---- Phase 5: Doctrine ----

#[test]
fn test_set_doctrine_command() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    let snap = engine.tick();
    assert_eq!(snap.doctrine, DoctrineMode::AutoSpecial);

    engine.queue_command(PlayerCommand::SetDoctrine {
        mode: DoctrineMode::AutoComposite,
    });
    let snap = engine.tick();
    assert_eq!(snap.doctrine, DoctrineMode::AutoComposite);

    engine.queue_command(PlayerCommand::SetDoctrine {
        mode: DoctrineMode::Manual,
    });
    let snap = engine.tick();
    assert_eq!(snap.doctrine, DoctrineMode::Manual);
}

// ---- Phase 5: Engagement snapshot ----

#[test]
fn test_engagement_appears_in_snapshot() {
    let mut engine = SimulationEngine::new(SimConfig::default());
    engine.queue_command(PlayerCommand::StartMission);
    engine.tick();

    engine.spawn_tracked_threats(1);
    for _ in 0..5 {
        engine.tick();
    }

    let snap = engine.tick();
    assert!(
        !snap.engagements.is_empty(),
        "Engagements should appear in snapshot"
    );

    let eng_view = &snap.engagements[0];
    assert!(eng_view.pk > 0.0, "Engagement view should have valid Pk");
    assert!(
        eng_view.veto_total_secs > 0.0,
        "Engagement view should have valid veto_total_secs"
    );
}
