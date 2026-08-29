//! Developer-authorized LM Studio native-v1 verification. Never runs by default.

use aiengineroom_lib::runtime::lm_studio::{LmStudioAdapter, LmStudioState};

fn required(name: &str) -> String {
    std::env::var(name)
        .unwrap_or_else(|_| panic!("explicit live-test prerequisite {name} is required"))
}

#[test]
#[ignore = "live LM Studio verification: requires four explicit AER_1L variables and may make one fixed synthetic inference"]
fn live_lm_studio_native_v1_observation_is_bounded() {
    let model = required("AER_1L_LM_STUDIO_MODEL");
    assert_eq!(required("AER_1L_ALLOW_LM_STUDIO_INFERENCE"), "1");
    assert_eq!(
        required("AER_1L_ACKNOWLEDGE_LM_STUDIO_JIT_SIDE_EFFECT"),
        "1"
    );
    assert_eq!(required("AER_1L_ALLOW_UNVERIFIED_COMPUTE_PLACEMENT"), "1");

    let adapter = LmStudioAdapter::new();
    let snapshot = tauri::async_runtime::block_on(adapter.snapshot());
    assert_eq!(snapshot.state, LmStudioState::Available);
    assert!(
        snapshot
            .models
            .iter()
            .any(|candidate| { candidate.model_id == model && candidate.inference_eligible }),
        "the developer-supplied model must be an LLM in the native-v1 catalogue"
    );

    // Production orchestration revalidates catalogue identity, then sends at
    // most one fixed, stateless native-v1 chat request. No output is logged.
    let observation = tauri::async_runtime::block_on(adapter.observe(&model, true, true));
    assert_eq!(observation.state, "completed");
    assert_eq!(observation.provider, "lm_studio");
    assert_eq!(observation.api_scope, "same_machine_loopback");
    assert_eq!(observation.compute_location, "not_independently_verified");
}
