# LM Studio live verification evidence

This records one owner-authorized live verification of the LM Studio native REST v1 integration on the Ubuntu 24.04 LTS x86_64 development environment. It is bounded integration evidence, not broad LM Studio compatibility certification, a benchmark, or proof of compute placement.

## Selected model and catalogue evidence

The owner deliberately selected the inference-eligible LLM `liquid/lfm2.5-1.2b`. LM Studio CLI identified it as a 1.2B-parameter `lfm2` model of approximately 1.25 GB and displayed its device/status as `Local`.

`GET http://127.0.0.1:1234/api/v1/models` exposed the selected model with:

- type `llm`;
- publisher `liquid`;
- key `liquid/lfm2.5-1.2b`;
- display name `Lfm2.5 1.2B`;
- architecture `lfm2`;
- quantization `Q8_0`;
- size `1246331840` bytes;
- parameter string `1.2B`;
- no loaded instances before the observation;
- maximum context length `128000`;
- format `gguf`;
- selected variant `liquid/lfm2.5-1.2b@q8_0`.

The same catalogue contained both LLM and embedding model types. The observation deliberately selected an LLM; the embedding entry was not selected. No broader model inventory, raw response, generated response, or prompt output is retained here.

## Authorization and result

The owner explicitly supplied all four test gates for the single run:

- `AER_1L_LM_STUDIO_MODEL` selected `liquid/lfm2.5-1.2b`;
- `AER_1L_ALLOW_LM_STUDIO_INFERENCE=1` authorized inference;
- `AER_1L_ACKNOWLEDGE_LM_STUDIO_JIT_SIDE_EFFECT=1` acknowledged possible JIT loading and later auto-unloading;
- `AER_1L_ALLOW_UNVERIFIED_COMPUTE_PLACEMENT=1` acknowledged that compute placement is not independently verified.

From `src-tauri`, the owner ran the ignored `lm_studio_live` integration test. The single test `live_lm_studio_native_v1_observation_is_bounded` passed: 1 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out, in 2.61 seconds. The owner then explicitly unset all four variables.

## Evidence boundary

This establishes that AI Engine Room reached the same-machine LM Studio server, the live native-v1 catalogue exposed the selected LLM, and the bounded explicitly authorized observation path completed successfully. The model did not need to be manually preloaded before the test, and the JIT-side-effect acknowledgement was explicitly supplied. Generated output was not evaluated for model quality, and the test was not a benchmark.

The result does not independently verify compute placement. The LM Studio CLI label `Local`, loopback API scope, downloaded state, or loaded state does not establish where computation physically occurred. AI Engine Room's compute-location state remains `not_independently_verified`. This evidence does not establish compatibility with all LM Studio models, configurations, versions, platforms, or machines. Windows and other hardware environments remain unverified.
