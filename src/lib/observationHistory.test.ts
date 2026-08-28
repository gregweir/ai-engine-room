import { describe, expect, it } from "vitest";
import { fixtureInferenceObservation } from "./fixtures/states";
import type { InferenceObservationView } from "./types";
import {
  OBSERVATION_HISTORY_LIMIT,
  appendCompletedObservation,
  comparisonSelection,
  pruneSelectedObservationIds,
  type ObservationHistoryItem,
} from "./observationHistory";

function observation(
  overrides: Partial<InferenceObservationView> = {},
): InferenceObservationView {
  return { ...fixtureInferenceObservation(), ...overrides };
}

describe("observation history", () => {
  it("prepends a valid completed observation and increments its session ID", () => {
    const source = observation();
    const result = appendCompletedObservation([], source, 1);
    expect(result).toEqual({
      history: [{ observation_id: 1, observation: source }],
      nextObservationId: 2,
      retained: true,
    });
    expect(result.history[0]?.observation).toBe(source);
  });

  it.each([
    { state: "timed_out" as const },
    { model: null },
    { model: "" },
    { model: "   " },
    { diagnostic_profile: null },
    { diagnostic_profile: "" },
    { diagnostic_profile: "   " },
  ])("rejects a result without completed identity: %o", (overrides) => {
    const existing = [{ observation_id: 4, observation: observation() }];
    const result = appendCompletedObservation(
      existing,
      observation(overrides),
      5,
    );
    expect(result.retained).toBe(false);
    expect(result.nextObservationId).toBe(5);
    expect(result.history).toEqual(existing);
    expect(result.history).not.toBe(existing);
  });

  it("caps at ten, keeps newest-first order, and evicts only the oldest", () => {
    let history: ObservationHistoryItem[] = [];
    let nextId = 1;
    for (let i = 0; i < OBSERVATION_HISTORY_LIMIT + 1; i += 1) {
      const result = appendCompletedObservation(
        history,
        observation({ generation_tokens_per_second: i + 0.5 }),
        nextId,
      );
      history = result.history;
      nextId = result.nextObservationId;
    }
    expect(history).toHaveLength(10);
    expect(history.map((item) => item.observation_id)).toEqual([
      11, 10, 9, 8, 7, 6, 5, 4, 3, 2,
    ]);
    expect(
      history.map((item) => item.observation.generation_tokens_per_second),
    ).toEqual([10.5, 9.5, 8.5, 7.5, 6.5, 5.5, 4.5, 3.5, 2.5, 1.5]);
  });

  it("does not mutate the input array or observation", () => {
    const source = observation();
    const existing = [{ observation_id: 1, observation: observation() }];
    const snapshot = [...existing];
    const sourceSnapshot = { ...source };
    const result = appendCompletedObservation(existing, source, 2);
    expect(existing).toEqual(snapshot);
    expect(source).toEqual(sourceSnapshot);
    expect(result.history).not.toBe(existing);
  });

  it("preserves every source field without adding private/action metadata", () => {
    const source = observation({
      prompt_eval_count: 0,
      eval_count: null,
      eval_duration_ns: 0,
      generation_tokens_per_second: null,
    });
    const retained = appendCompletedObservation([], source, 1).history[0]!;
    expect(retained.observation).toEqual(source);
    expect(Object.keys(retained)).toEqual(["observation_id", "observation"]);
    const serialized = JSON.stringify(retained);
    expect(serialized).not.toMatch(
      /acknowledged|prompt"|response|raw_error|endpoint|timestamp|authorization/i,
    );
  });

  it("prunes stale selected IDs and preserves at most two retained IDs", () => {
    const history = [1, 2, 3].map((id) => ({
      observation_id: id,
      observation: observation(),
    }));
    expect(pruneSelectedObservationIds([7, 3, 2, 1], history)).toEqual([3, 2]);
  });

  it("allows same-profile pairs for the same or different models", () => {
    const sameModel = [
      { observation_id: 1, observation: observation() },
      { observation_id: 2, observation: observation() },
    ];
    expect(comparisonSelection([1, 2], sameModel).kind).toBe("eligible");

    const differentModels = [
      { observation_id: 1, observation: observation({ model: "example:a" }) },
      { observation_id: 2, observation: observation({ model: "example:b" }) },
    ];
    expect(comparisonSelection([1, 2], differentModels).kind).toBe("eligible");
  });

  it("blocks cross-provider comparison even with the same profile", () => {
    const history = [
      { observation_id: 1, observation: observation({ provider: "ollama" }) },
      {
        observation_id: 2,
        observation: observation({ provider: "lm_studio" }),
      },
    ];
    expect(comparisonSelection([1, 2], history).kind).toBe("provider_mismatch");
  });

  it("rejects mismatched, missing, and stale profile comparisons", () => {
    const mismatched = [
      { observation_id: 1, observation: observation() },
      {
        observation_id: 2,
        observation: observation({ diagnostic_profile: "diagnostic-other" }),
      },
    ];
    expect(comparisonSelection([1, 2], mismatched).kind).toBe(
      "profile_mismatch",
    );

    const missing = [
      { observation_id: 1, observation: observation() },
      {
        observation_id: 2,
        observation: observation({ diagnostic_profile: null }),
      },
    ];
    expect(comparisonSelection([1, 2], missing).kind).toBe("profile_mismatch");
    expect(comparisonSelection([1, 99], mismatched).kind).toBe("incomplete");
  });
});
