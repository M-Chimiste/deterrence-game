#[cfg(test)]
mod tests {
    use deterrence_core::constants::*;
    use deterrence_core::enums::{ThreatArchetype, ThreatPhase};
    use deterrence_core::types::{Position, Velocity};

    use crate::fsm::{evaluate, ThreatContext};

    fn make_context(
        archetype: ThreatArchetype,
        phase: ThreatPhase,
        range: f64,
        is_engaged: bool,
        elapsed: f64,
    ) -> ThreatContext {
        // Place threat at (0, range, altitude) heading toward origin
        let altitude = match archetype {
            ThreatArchetype::SeaSkimmerMk1 => SEA_SKIMMER_ALTITUDE,
            ThreatArchetype::SeaSkimmerMk2 => SEA_SKIMMER_ALTITUDE * 0.8,
            ThreatArchetype::SupersonicCruiser => 5000.0,
            ThreatArchetype::SubsonicDrone => 3000.0,
            ThreatArchetype::TacticalBallistic => 30_000.0,
        };
        let speed = match archetype {
            ThreatArchetype::SeaSkimmerMk1 => SEA_SKIMMER_SPEED,
            ThreatArchetype::SeaSkimmerMk2 => SEA_SKIMMER_SPEED * 1.1,
            ThreatArchetype::SupersonicCruiser => SUPERSONIC_CRUISER_SPEED,
            ThreatArchetype::SubsonicDrone => 100.0,
            ThreatArchetype::TacticalBallistic => 1500.0,
        };
        ThreatContext {
            archetype,
            phase,
            position: Position::new(0.0, range, altitude),
            velocity: Velocity::new(0.0, -speed, 0.0),
            target: Position::new(0.0, 0.0, 0.0),
            range_to_target: range,
            is_engaged,
            elapsed_in_phase_secs: elapsed,
        }
    }

    #[test]
    fn test_sea_skimmer_cruise_to_popup() {
        // At popup range, sea-skimmer should transition from Cruise to PopUp
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::Cruise,
            THREAT_POPUP_RANGE - 100.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::PopUp);
        // Should have positive z velocity (climbing)
        assert!(update.new_velocity.z > 0.0, "PopUp should climb");
    }

    #[test]
    fn test_sea_skimmer_no_popup_far_away() {
        // Far from target, sea-skimmer stays in Cruise
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::Cruise,
            100_000.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Cruise);
    }

    #[test]
    fn test_sea_skimmer_popup_to_terminal() {
        // After popup duration, transitions to Terminal
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::PopUp,
            40_000.0,
            false,
            THREAT_POPUP_DURATION_SECS + 0.1,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Terminal);
    }

    #[test]
    fn test_sea_skimmer_popup_stays_during_maneuver() {
        // Before popup duration, stays in PopUp
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::PopUp,
            45_000.0,
            false,
            1.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::PopUp);
    }

    #[test]
    fn test_sea_skimmer_terminal_to_impact() {
        // At impact range, transitions to Impact
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::Terminal,
            THREAT_IMPACT_RANGE - 10.0,
            false,
            5.0,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Impact);
    }

    #[test]
    fn test_supersonic_cruise_to_terminal() {
        // Supersonic cruiser goes directly from Cruise to Terminal (no popup)
        let ctx = make_context(
            ThreatArchetype::SupersonicCruiser,
            ThreatPhase::Cruise,
            THREAT_TERMINAL_RANGE - 100.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Terminal);
    }

    #[test]
    fn test_ballistic_terminal_dive() {
        // Tactical ballistic in terminal should have downward velocity
        let ctx = ThreatContext {
            archetype: ThreatArchetype::TacticalBallistic,
            phase: ThreatPhase::Cruise,
            position: Position::new(0.0, THREAT_TERMINAL_RANGE - 100.0, 30_000.0),
            velocity: Velocity::new(0.0, -1500.0, 0.0),
            target: Position::new(0.0, 0.0, 0.0),
            range_to_target: THREAT_TERMINAL_RANGE - 100.0,
            is_engaged: false,
            elapsed_in_phase_secs: 0.0,
        };
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Terminal);
        assert!(
            update.new_velocity.z < 0.0,
            "Ballistic terminal should dive"
        );
    }

    #[test]
    fn test_mk2_evasive_when_engaged() {
        // SeaSkimmerMk2 in Terminal with is_engaged should go Evasive
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk2,
            ThreatPhase::Terminal,
            15_000.0,
            true,
            2.0,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Evasive);
    }

    #[test]
    fn test_mk1_no_evasion() {
        // SeaSkimmerMk1 cannot evade even when engaged
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::Terminal,
            15_000.0,
            true,
            2.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Terminal);
    }

    #[test]
    fn test_drone_stays_cruise_until_impact() {
        // SubsonicDrone has no terminal phase, stays Cruise
        let ctx = make_context(
            ThreatArchetype::SubsonicDrone,
            ThreatPhase::Cruise,
            5_000.0, // past terminal range but drone doesn't use it
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Cruise);
    }

    #[test]
    fn test_drone_impact() {
        // SubsonicDrone impacts at close range from Cruise
        let ctx = make_context(
            ThreatArchetype::SubsonicDrone,
            ThreatPhase::Cruise,
            THREAT_IMPACT_RANGE - 10.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Impact);
    }

    #[test]
    fn test_destroyed_no_transition() {
        // Destroyed phase is terminal — no further transitions
        let ctx = make_context(
            ThreatArchetype::SeaSkimmerMk1,
            ThreatPhase::Destroyed,
            1000.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Destroyed);
    }

    #[test]
    fn test_impact_no_transition() {
        // Impact phase is terminal — no further transitions
        let ctx = make_context(
            ThreatArchetype::SupersonicCruiser,
            ThreatPhase::Impact,
            10.0,
            false,
            0.0,
        );
        let update = evaluate(&ctx);
        assert!(!update.phase_changed);
        assert_eq!(update.new_phase, ThreatPhase::Impact);
    }
}
