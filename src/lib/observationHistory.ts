import type { InferenceObservationView } from "./types";

export const OBSERVATION_HISTORY_LIMIT = 10;
export const COMPARISON_LIMITATION =
  "These are individual observed runs, not benchmark averages. Execution location is not determined, so differences may reflect different execution environments. Model loading and runtime state may also differ between runs. Treat these values as run-specific observations, not general performance ratings.";
export const PROFILE_MISMATCH =
  "Direct comparison is unavailable because these observations used different or unknown diagnostic profiles. Review each observation separately.";
export const PROVIDER_MISMATCH =
  "Direct comparison is unavailable because these observations came from different runtime providers with different reporting semantics. Review each observation separately.";
export const LOAD_DURATION_LIMITATION =
  "Ollama-reported load duration can depend on whether the model was already loaded before an observation. Engine Room does not infer a warm or cold state from this value.";
export const GENERATED_TOKEN_LIMITATION =
  "Generated-token counts may differ between observations, so duration values may cover different amounts of generated output.";

export interface ObservationHistoryItem {
  observation_id: number;
  observation: InferenceObservationView;
}

export interface AppendObservationResult {
  history: ObservationHistoryItem[];
  nextObservationId: number;
  retained: boolean;
}

export type ComparisonSelection =
  | { kind: "incomplete" }
  | {
      kind: "profile_mismatch";
      first: ObservationHistoryItem;
      second: ObservationHistoryItem;
    }
  | {
      kind: "provider_mismatch";
      first: ObservationHistoryItem;
      second: ObservationHistoryItem;
    }
  | {
      kind: "eligible";
      first: ObservationHistoryItem;
      second: ObservationHistoryItem;
    };

function hasIdentity(observation: InferenceObservationView): boolean {
  return (
    observation.state === "completed" &&
    typeof observation.model === "string" &&
    observation.model.trim().length > 0 &&
    typeof observation.diagnostic_profile === "string" &&
    observation.diagnostic_profile.trim().length > 0
  );
}

export function appendCompletedObservation(
  history: readonly ObservationHistoryItem[],
  observation: InferenceObservationView,
  nextObservationId: number,
): AppendObservationResult {
  if (!hasIdentity(observation)) {
    return {
      history: [...history],
      nextObservationId,
      retained: false,
    };
  }

  return {
    history: [
      { observation_id: nextObservationId, observation },
      ...history,
    ].slice(0, OBSERVATION_HISTORY_LIMIT),
    nextObservationId: nextObservationId + 1,
    retained: true,
  };
}

export function pruneSelectedObservationIds(
  selectedIds: readonly number[],
  history: readonly ObservationHistoryItem[],
): number[] {
  const retainedIds = new Set(history.map((item) => item.observation_id));
  return selectedIds.filter((id) => retainedIds.has(id)).slice(0, 2);
}

export function comparisonSelection(
  selectedIds: readonly number[],
  history: readonly ObservationHistoryItem[],
): ComparisonSelection {
  if (selectedIds.length !== 2) return { kind: "incomplete" };

  const first = history.find((item) => item.observation_id === selectedIds[0]);
  const second = history.find((item) => item.observation_id === selectedIds[1]);
  if (!first || !second) return { kind: "incomplete" };

  const firstProfile = first.observation.diagnostic_profile;
  const secondProfile = second.observation.diagnostic_profile;
  if (
    first.observation.state !== "completed" ||
    second.observation.state !== "completed" ||
    typeof firstProfile !== "string" ||
    firstProfile.length === 0 ||
    typeof secondProfile !== "string" ||
    secondProfile.length === 0 ||
    firstProfile !== secondProfile
  ) {
    return { kind: "profile_mismatch", first, second };
  }

  if (
    (first.observation.provider ?? "ollama") !==
    (second.observation.provider ?? "ollama")
  ) {
    return { kind: "provider_mismatch", first, second };
  }

  return { kind: "eligible", first, second };
}
