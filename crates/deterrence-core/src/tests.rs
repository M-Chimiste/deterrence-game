#[cfg(test)]
mod tests {
    use crate::commands::PlayerCommand;
    use crate::enums::*;
    use crate::events::{Alert, AudioEvent};
    use crate::state::GameStateSnapshot;
    use crate::types::{Position, SimTime, Velocity};

    /// Verify all enums round-trip through serde_json.
    #[test]
    fn test_classification_serde() {
        let variants = vec![
            Classification::Unknown,
            Classification::Pending,
            Classification::AssumedFriend,
            Classification::Friend,
            Classification::Neutral,
            Classification::Suspect,
            Classification::Hostile,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: Classification = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn test_doctrine_mode_serde() {
        let variants = vec![
            DoctrineMode::Manual,
            DoctrineMode::AutoSpecial,
            DoctrineMode::AutoComposite,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: DoctrineMode = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn test_weapon_type_serde() {
        let variants = vec![
            WeaponType::Standard,
            WeaponType::ExtendedRange,
            WeaponType::PointDefense,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: WeaponType = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn test_engagement_phase_serde() {
        let variants = vec![
            EngagementPhase::SolutionCalc,
            EngagementPhase::Ready,
            EngagementPhase::Launched,
            EngagementPhase::Midcourse,
            EngagementPhase::Terminal,
            EngagementPhase::Intercept,
            EngagementPhase::Complete,
            EngagementPhase::Aborted,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).unwrap();
            let back: EngagementPhase = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
    }

    #[test]
    fn test_cell_status_serde() {
        let variants = vec![
            CellStatus::Ready(WeaponType::Standard),
            CellStatus::Assigned(WeaponType::ExtendedRange),
            CellStatus::Expended,
            CellStatus::Empty,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: CellStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back);
        }
    }

    /// Verify PlayerCommand round-trips through serde (tagged union).
    #[test]
    fn test_player_command_serde() {
        let commands = vec![
            PlayerCommand::HookTrack { track_number: 42 },
            PlayerCommand::UnhookTrack,
            PlayerCommand::ClassifyTrack {
                track_number: 7,
                classification: Classification::Hostile,
            },
            PlayerCommand::VetoEngagement { engagement_id: 1 },
            PlayerCommand::ConfirmEngagement { engagement_id: 2 },
            PlayerCommand::SetTimeScale { scale: 2.0 },
            PlayerCommand::Pause,
            PlayerCommand::Resume,
            PlayerCommand::StartMission,
        ];
        for cmd in &commands {
            let json = serde_json::to_string(cmd).unwrap();
            let back: PlayerCommand = serde_json::from_str(&json).unwrap();
            // Compare JSON representations since PlayerCommand doesn't derive PartialEq
            assert_eq!(json, serde_json::to_string(&back).unwrap());
        }
    }

    /// Verify AudioEvent round-trips through serde.
    #[test]
    fn test_audio_event_serde() {
        let events = vec![
            AudioEvent::NewContact {
                bearing: 1.5,
                track_number: 101,
            },
            AudioEvent::BirdAway {
                weapon_type: WeaponType::Standard,
            },
            AudioEvent::Splash {
                result: InterceptResult::Hit,
                track_number: 42,
            },
            AudioEvent::VampireImpact { bearing: 3.14 },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let _back: AudioEvent = serde_json::from_str(&json).unwrap();
        }
    }

    /// Verify Alert round-trips through serde.
    #[test]
    fn test_alert_serde() {
        let alert = Alert {
            level: AlertLevel::Critical,
            message: "VAMPIRE VAMPIRE".to_string(),
            tick: 1000,
        };
        let json = serde_json::to_string(&alert).unwrap();
        let back: Alert = serde_json::from_str(&json).unwrap();
        assert_eq!(alert.message, back.message);
        assert_eq!(alert.tick, back.tick);
    }

    /// Verify GameStateSnapshot can be serialized to JSON.
    #[test]
    fn test_snapshot_serde() {
        let snapshot = GameStateSnapshot::default();
        let json = serde_json::to_string(&snapshot).unwrap();
        let back: GameStateSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.time.tick, back.time.tick);
        assert_eq!(snapshot.phase, back.phase);
        // Verify the default snapshot is reasonably small
        assert!(
            json.len() < 1024,
            "Empty snapshot should be <1KB, was {} bytes",
            json.len()
        );
    }

    /// Verify Position geometry calculations.
    #[test]
    fn test_position_range() {
        let a = Position::new(0.0, 0.0, 0.0);
        let b = Position::new(3.0, 4.0, 0.0);
        assert!((a.range_to(&b) - 5.0).abs() < 1e-10);
        assert!((a.horizontal_range_to(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_position_bearing() {
        let origin = Position::new(0.0, 0.0, 0.0);

        // Due North (positive Y)
        let north = Position::new(0.0, 100.0, 0.0);
        assert!((origin.bearing_to(&north) - 0.0).abs() < 1e-10);

        // Due East (positive X)
        let east = Position::new(100.0, 0.0, 0.0);
        let expected_east = std::f64::consts::FRAC_PI_2;
        assert!(
            (origin.bearing_to(&east) - expected_east).abs() < 1e-10,
            "East bearing should be PI/2, got {}",
            origin.bearing_to(&east)
        );
    }

    /// Verify Velocity calculations.
    #[test]
    fn test_velocity_speed() {
        let v = Velocity::new(3.0, 4.0, 0.0);
        assert!((v.speed() - 5.0).abs() < 1e-10);
        assert!((v.horizontal_speed() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_velocity_heading() {
        // Heading north (positive Y)
        let north = Velocity::new(0.0, 10.0, 0.0);
        assert!((north.heading() - 0.0).abs() < 1e-10);

        // Heading east (positive X)
        let east = Velocity::new(10.0, 0.0, 0.0);
        let expected = std::f64::consts::FRAC_PI_2;
        assert!((east.heading() - expected).abs() < 1e-10);
    }

    /// Verify SimTime advancement.
    #[test]
    fn test_sim_time_advance() {
        let mut time = SimTime::default();
        assert_eq!(time.tick, 0);
        assert_eq!(time.elapsed_secs, 0.0);

        for _ in 0..30 {
            time.advance();
        }
        assert_eq!(time.tick, 30);
        // 30 ticks at 30Hz = 1 second
        assert!((time.elapsed_secs - 1.0).abs() < 1e-10);
    }
}
