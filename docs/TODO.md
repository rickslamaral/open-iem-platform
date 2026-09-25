## 2026-09-25 — Batch Phase 413–422 streaming registry boundaries
- [x] Cobrir remoção desconhecida/exata por usuário e device, limpeza seletiva e independência de sessões bound/unbound.
- [x] Cobrir requeue/drain parcial, FIFO, preservação de metadados e replacement bound sem duplicação.
- [x] Cobrir idempotência de remoção após replacement e independência entre fila de transporte e registry.
- Evidência CODE local: 280 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-25 — Batch Phase 403–412 streaming registry boundaries
- [x] Cobrir budget zero, exato e oversized na fila de transporte, preservando FIFO e saída existente após remoção de sessão.
- [x] Cobrir requeue vazio, ordenação determinística para quatro sessões, replacement sem duplicação e remoção seletiva por device.
- [x] Cobrir rejeição de registro duplicado inválido e mix ID oversized sem substituir sessão válida.
- Evidência CODE local: 270 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.


## 2026-09-25 — Batch Phase 393–402 streaming registry and transport boundaries
- [x] Cobrir dez fronteiras de replacement, remoção, ordenação determinística e fila de transporte no crate `streaming`.
- Evidência CODE local: 260 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 383–392 streaming state-preservation boundaries
- [x] Cobrir whitespace user ID, candidate oversized/whitespace, user ID oversized em ICE e falhas de offer sem mutação.
- [x] Cobrir fingerprint incompatível, remoção de device inválida e preservação de metadados/estado existente.
- Evidência CODE local: 250 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 373–382 streaming input and identity boundaries
- [x] Cobrir rejeição sem mutação para user ID vazio/oversized, mix ID oversized e SDP oversized.
- [x] Cobrir candidato vazio, user ID whitespace-only e sessão inexistente sem mutação.
- [x] Cobrir identidade revogada e bindings de usuário/mix incompatíveis antes da criação de sessão.
- Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 353–362 streaming registry boundaries
- [x] Cobrir dez fronteiras adicionais de drain/requeue de transporte, remoção de device, ordenação determinística e ciclo de vida de sessões no crate `streaming`.
- Evidência: 220 testes `streaming` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 343–352 streaming transport and registry boundaries
- [x] Cobrir drain/requeue bounded de transporte: budget zero, budget limitado, budget oversized, fila vazia e preservação de ordem.
- [x] Cobrir remoção de dispositivo desconhecido, remoção seletiva, substituição bound, ordem determinística e independência entre remoção de sessão e fila de transporte.
- Evidência CODE local: 210 testes `streaming` PASS (`cargo test --manifest-path server/Cargo.toml -p streaming --lib`); runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 321–330 streaming registry boundaries
- [x] Cobrir contagem e transições de vazio da `SessionRegistry`.
- [x] Cobrir remoção idempotente por `device_id` e preservação sem correspondência.
- [x] Cobrir metadados bound de dispositivo e substituição do mesmo usuário.
- [x] Cobrir drenagem/requeue de transporte com budget zero, fila vazia, ordem e preservação após remoção.
- Evidência CODE local: 200 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 311–320 streaming registry boundaries
- [x] Cobrir listagem, remoção idempotente e metadados de sessões negociadas.
- [x] Cobrir drain/requeue bounded de datagrams: budget zero, ordem FIFO, fila vazia e budget parcial.
- Evidência CODE local: 190 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 301–310 streaming budget and transport boundaries
- [x] Cobrir zero-budget e budget oversized em `MediaSession`/`MediaPlane` sem consumir frames indevidos.
- [x] Cobrir rejeição fail-closed para sessão ausente e mix inválido sem mutação.
- [x] Cobrir drenagem bounded da `MediaBridge` com fila vazia, sem subscribers e múltiplas chamadas.
- [x] Cobrir `TransportAdapter::send` com budget zero e limite máximo de datagrams.
- Evidência CODE local: 180 testes `streaming` PASS; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 291–300
- [x] Cobrir 10 fronteiras de autorização, fan-out, overflow e pairing no crate `streaming`; evidência CODE local.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Batch Phase 281–290 streaming boundaries
- [x] Cobrir contrato de metadados, budgets e overflow de `MediaSession`.
- [x] Cobrir drain bounded, sessão ausente e registro duplicado de `MediaPlane`.
- [x] Rejeitar user ID whitespace-only e candidato ICE terminado em newline antes de mutação.
- Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.


## 2026-09-24 — Phase 280 bounded bridge backlog exhaustion
- [x] Cobrir drenagem sucessiva com `output_budget == 1` e `frame_budget == usize::MAX`: frames excedentes da `MediaBridge` permanecem disponíveis até esgotamento. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 279 bridge drain budget boundary
- [x] Cobrir que `SessionRegistry::drive_once` limita drenagem da `MediaBridge` ao orçamento compartilhado antes do fan-out; chamada seguinte entrega frame preservado. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 277 negotiated MediaBridge shared output budget

- [x] Limitar drenagem de frames de sessões negociadas ao `output_budget` compartilhado, preservando frames já roteados para chamadas posteriores quando há mais frames na fila que capacidade de codificação.
- Evidência: teste `drive_once_limits_drain_to_shared_output_budget` e clippy do crate `streaming` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 276 negotiated MediaBridge frame preservation
- [x] Cobrir retenção de frames em sessão negociada sem mídia de áudio: teste confirma que frames permanecem na fila quando `media_mid` ainda não existe. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi 5 permanecem não validados.

## 2026-09-24 — Phase 275 negotiated MediaBridge zero-output preservation

- [x] Cobrir `SessionRegistry::drive_once` com sessão negociada e `output_budget == 0`: nenhum frame é consumido; chamada posterior entrega o frame preservado.
- Evidência: teste `drive_once_zero_output_budget_preserves_negotiated_bridge_frames` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 274 MediaBridge recovery drop accounting
- [x] Cobrir entrega após recuperação de fila: frame aceito depois de um overflow não incrementa novamente contadores de descarte; evidência CODE local.

## 2026-09-24 — Phase 273 MediaBridge bounded order/timestamp preservation

- [x] Teste bounded drain em chamadas sucessivas preserva ordem FIFO, revisão e `capture_timestamp`; evidência CODE local.

## 2026-09-24 — Phase 271 MediaBridge repeated overflow rejection
- [x] Cobrir rejeições repetidas enquanto a fila está cheia; nenhum frame rejeitado reaparece após recuperação e a ordem dos frames aceitos permanece intacta.
- Evidência: teste `rejected_frame_is_not_enqueued_after_queue_recovers` ampliado; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 270 MediaBridge bounded drain without subscribers
- [x] Cobrir drenagens sucessivas com budget unitário sem sessões: cada chamada consome somente um frame e a bridge termina vazia.
- Evidência: teste `bounded_drain_without_sessions_preserves_remaining_frames`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 269 MediaBridge rejected-frame recovery
- [x] Cobrir que frame rejeitado por fila cheia não reaparece após recuperação; frames aceitos preservam ordem e novo frame entra no fim da fila.
- Evidência: teste `rejected_frame_is_not_enqueued_after_queue_recovers`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 268 MediaBridge zero-drop success assertion
- [x] Cobrir entrega normal sem overflow: `MediaPlane::total_dropped()` permanece zero após drenagem bem-sucedida.
- Evidência: testes `media_bridge` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 267 MediaBridge aggregate drop accessor assertion
- [x] Cobrir `MediaPlane::total_dropped()` no caminho de overflow da `MediaBridge`, além do contador atômico interno e contador por sessão.
- Evidência: teste `drain_consumes_frames_when_destination_queue_is_full` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 266 MediaBridge per-session drop counter assertion
- [x] Cobrir que overflow de fila de destino incrementa `MediaSession::drop_count` além do contador agregado.
- Evidência: teste `drain_consumes_frames_when_destination_queue_is_full` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 265 MediaBridge no-subscriber drain boundary
- [x] Cobrir drenagem sem sessões registradas: frames são consumidos sem criar estado e bridge fica vazia.
- Evidência: teste `drain_consumes_frames_without_registered_sessions` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 264 MediaBridge oversized budget boundary
- [x] Cobrir budget acima dos frames disponíveis: `MediaBridge::drain_to_with_budget` roteia todos os frames disponíveis, preserva ordem e retorna zero na drenagem seguinte.
- Evidência: teste `oversized_budget_routes_all_available_frames` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 263 MediaBridge overflow preservation assertions
- [x] Fortalecer teste de overflow para verificar revisões exatas dos frames preservados, não apenas capacidade e contagem.
- Evidência: teste `drain_consumes_frames_when_destination_queue_is_full` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 262 MediaBridge destination overflow accounting
- [x] Cobrir consumo da bridge quando fila de sessão está cheia: frame é consumido, descarte contado e frames já enfileirados preservados.
- Evidência: teste `drain_consumes_frames_when_destination_queue_is_full` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 261 MediaBridge empty drain boundary
- [x] Cobrir `MediaBridge::drain_to` sem frames: retorna zero e não altera sessões.
- Evidência: teste `empty_drain_returns_zero_without_touching_media_plane` e suíte Rust completa PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.
## 2026-09-23 — Phase 260 MediaBridge capture timestamp preservation
- [x] Cobrir propagação de `capture_timestamp` através de `MediaBridge::drain_to`.
- Evidência: teste `drain_preserves_capture_timestamp` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 259 MediaBridge zero-budget preservation
- [x] Cobrir budget zero em `MediaBridge::drain_to_with_budget`: não consome frames enfileirados e preserva entrega posterior.
- Evidência: teste `zero_budget_preserves_queued_frames` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 258 MediaBridge budget preservation
- [x] Cobrir drenagem parcial de `MediaBridge`: budget roteia somente frames solicitados e preserva ordem para chamada seguinte.
- Evidência: teste `drain_with_budget_routes_only_requested_frames` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 256 session removal and re-registration boundary
- [x] Cobrir que remoção interrompe entrega e re-registro inicia fila e sequência limpas.
- Evidência: teste `remove_session_stops_delivery_and_reregister_starts_clean_session` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 254 multibyte user ID oversized byte-boundary coverage
- [x] Cobrir user ID UTF-8 multibyte acima de `MAX_MEDIA_USER_ID_BYTES`; rejeição preserva registry.
- Evidência: teste `register_rejects_multibyte_user_id_above_byte_limit_without_mutation`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 253 multibyte user ID byte-boundary coverage
- [x] Cobrir user ID UTF-8 multibyte exatamente em `MAX_MEDIA_USER_ID_BYTES`; validação usa bytes e aceita limite exato.
- Evidência: teste `register_accepts_multibyte_user_id_at_exact_byte_limit` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 252 multi-session overflow accounting
- [x] Cobrir contagem de descarte por sessão e agregada quando fan-out encontra filas cheias.
- Evidência: teste `overflow_drop_accounting_matches_each_full_session`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 251 overflow sequence gap coverage
- [x] Cobrir sequência após overflow: frames aceitos preservam sequência e próximo frame expõe salto esperado.
- Evidência: teste `overflowed_frame_sequence_reports_gap_after_drain` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 250 missing media-session removal boundary
- [x] Cobrir remoção de sessão inexistente: retorna `false` e preserva sessão ativa no registry.
- Evidência: teste `remove_missing_session_returns_false_without_mutation` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 248 media-plane drop accounting coverage
- [x] Cobrir contagem exata de frames descartados por overflow, fila preservada e sequência sem lacunas nos frames aceitos.
- Phase 247 também concluída: budget acima da capacidade retorna apenas frames disponíveis, preserva ordem e não inventa frames.
- Evidência: testes `push_frame_output_overflow_increments_drop_count` e `drain_session_frames_caps_oversized_budget_to_available_frames`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 246 media-plane multi-session fan-out coverage
- [x] Cobrir fan-out simultâneo para sessões em mix slots distintos, incluindo samples, revision e capture timestamp.
- Evidência: teste `push_frame_output_fans_out_each_session_mix` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 245 selective media-session removal coverage
- [x] Cobrir remoção de uma sessão sem apagar sessões vizinhas; segunda remoção retorna `false` e preserva registry.
- Evidência: teste `remove_session_preserves_other_sessions` e suíte Rust completa PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — Phase 242 whitespace-only media-session user ID boundary
- [x] Rejeitar user ID composto apenas por whitespace antes de criar sessão de mídia e preservar registry.
- Evidência: teste `register_rejects_whitespace_only_user_id_without_mutation` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 240 media-session user ID boundary)
- [x] Rejeitar user ID vazio ou acima de `MAX_MEDIA_USER_ID_BYTES` antes de criar sessão de mídia; aceitar exatamente no limite e preservar registry em rejeições.
- Evidência: 15 testes `media_plane` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — malformed trickle ICE mutation guard
- [x] Cobrir rejeição de candidato ICE malformado após sessão existente sem alterar registry.
- Evidência: teste `malformed_candidate_rejected_without_registry_change` PASS localmente; runtime WebRTC/DTLS-SRTP, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — malformed offer replacement guard
- [x] Cobrir rejeição de SDP malformado dentro do limite sem substituir sessão já negociada.
- Evidência: teste `invalid_offer_rejected_without_replacing_existing_session` PASS localmente; runtime WebRTC/DTLS-SRTP, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — oversized offer replacement guard
- [x] Cobrir rejeição de SDP acima de `MAX_SDP_BYTES` sem substituir sessão já negociada.
- Evidência: teste `oversized_offer_rejected_without_replacing_existing_session` PASS localmente; runtime WebRTC/DTLS-SRTP, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — DTLS fingerprint parser boundary coverage
- [x] Cobrir canonicalização case-insensitive e rejeição de fingerprint DTLS malformado.
- Evidência: testes unitários `dtls_fingerprint_is_canonicalized_case_insensitively` e `malformed_dtls_fingerprint_is_rejected` PASS localmente; runtime WebRTC/DTLS-SRTP, rede real e Raspberry Pi permanecem não validados.

## 2026-09-23 — DTLS fingerprint binding rejection coverage
- [x] Cobrir fingerprint DTLS incompatível: oferta rejeitada e registry preservado, sem criar sessão parcial.
- Evidência: teste unitário `bound_session_rejects_mismatched_fingerprint_without_mutating_registry` PASS localmente; runtime WebRTC/DTLS-SRTP, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (PairingRegistry credential boundary coverage)
- [x] Cobrir credential exatamente em MAX_CREDENTIAL_BYTES e rejeitar credential acima do limite sem alterar registry.
- Evidência: testes unitários `credential_at_maximum_length_is_accepted` e `oversized_credential_is_rejected_without_registry_change` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 232 maximum trickle ICE candidate length boundary)
- [x] Cobrir candidate exatamente em MAX_CANDIDATE_BYTES: candidato válido aceito e sessão preservada.
- Evidência: teste unitário candidate_at_maximum_length_is_accepted PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 231 maximum SDP length boundary)
- [x] Cobrir SDP exatamente em MAX_SDP_BYTES: oferta válida aceita sem criar caminho alternativo de validação.
- Evidência: teste unitário offer_at_maximum_sdp_length_is_accepted PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 230 maximum trickle ICE user ID boundary)
- [x] Cobrir user_id exatamente em MAX_USER_ID_BYTES em add_ice_candidate: candidato válido aceito e sessão preservada.
- Evidência: teste unitário maximum_length_candidate_user_id_is_accepted PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 229 oversized trickle ICE user ID boundary)
- [x] Cobrir user_id acima de MAX_USER_ID_BYTES em add_ice_candidate: candidato rejeitado e sessão existente preservada.
- Evidência: teste unitário oversized_candidate_user_id_rejected_without_registry_change PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 228 oversized mix ID boundary)
- [x] Cobrir mix_id acima de MAX_MIX_ID_BYTES: negociação rejeitada e registry permanece sem sessão.
- Evidência: teste unitário mix_id_above_maximum_length_is_rejected PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 227 oversized user ID boundary)
- [x] Cobrir user_id acima de MAX_USER_ID_BYTES: negociação rejeitada e registry permanece sem sessão.
- Evidência: teste unitário user_id_above_maximum_length_is_rejected PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (API device revocation session binding coverage)
- [x] Cobrir revogação de dispositivo conhecido: remove sessões vinculadas ao dispositivo revogado e preserva outra sessão ativa.
- Evidência: teste de integração `revoke_device_removes_bound_session_preserves_unbound_session`; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 225 unknown revocation preservation)
- [x] Cobrir revogação de dispositivo inexistente: resposta `404` preserva sessões de streaming ativas.
- Evidência: teste de integração `revoke_nonexistent_device_preserves_active_sessions`; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-23 (Phase 224 session removal boundary coverage)
- [x] Cobrir `SessionRegistry::remove` quando usuário não possui sessão; operação retorna `false` e preserva registry vazio.
- Evidência: teste unitário `remove_returns_false_for_missing_session`; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-23 (Phase 223 transport send budget coverage)
- [x] Cobrir `TransportAdapter::send` com budget acima do limite: envia no máximo `TRANSPORT_SEND_BUDGET`; o iterador genérico não expõe o restante ao chamador.
- Evidência: teste unitário `send_caps_oversized_budget` PASS localmente; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-23 (transport send budget boundary coverage)

- [x] Cobrir `TransportAdapter::send_from_registry` com budget acima do limite: envia no máximo `TRANSPORT_SEND_BUDGET` e preserva sufixo na fila.
- Evidência: teste unitário `transport_adapter_caps_oversized_registry_budget` PASS localmente; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-23 (transport queue overflow coverage)

- [x] Cobrir requeue de datagrama quando fila de transporte está cheia; descarte permanece bounded e fila mantém capacidade máxima.
- Evidência: teste unitário `requeue_transport_outputs_drops_when_queue_is_full` PASS localmente; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-22 (Phases 219-221 drive_once budget edge coverage)

- [x] Phases 219-221 — cobrir budget de frames zero, sessões sem mídia negociada e registry sem sessões em `SessionRegistry::drive_once`.
- Evidência: suíte server completa PASS localmente; classificação CODE local. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA físico e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-22 (Phase 215 deterministic six-fault reconnect coverage)

- [x] Phase 215 — cobrir reconnect após bandwidth, outage, loss, jitter, reorder e duplicate determinísticos no OpusReceiver.
- Evidência: 92 testes headless_receiver PASS localmente; classificação CODE local. Rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA físico e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-22 (Phases 208-214 deterministic combined receiver coverage)

- [x] Phases 208-214 — cobrir cinco combinações quad-fault, duas penta-fault e uma hexa-fault sem reconnect no `OpusReceiver`.
- Evidência: 91 testes `headless_receiver` PASS localmente; classificação CODE local. Rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA físico e Raspberry Pi 5 permanecem não validados.

## Estado atual - 2026-09-22 (Phases 187-207 receiver reconnect fault coverage)

- [x] Phases 187-192 — completar combinações triple-fault com reconnect: outage+jitter+reorder, bandwidth+reorder+duplicate, loss+jitter+reorder, loss+jitter+duplicate, loss+reorder+duplicate e jitter+reorder+duplicate.
- [x] Phases 193-197 — completar combinações quad-fault sem outage: bandwidth+loss+jitter+reorder, bandwidth+loss+jitter+duplicate, bandwidth+loss+reorder+duplicate, bandwidth+jitter+reorder+duplicate e loss+jitter+reorder+duplicate.
- [x] Phases 198-203 — combinações outage+quad-fault com reconnect.
- [x] Phases 204-207 — quatro combinações penta-fault com reconnect.
- Evidência: 83 testes `headless_receiver` e 66 testes unitários PASS em CI; classificação CODE/CI. Rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA físico e Raspberry Pi 5 permanecem não validados.
- Phases 208-214 sem reconnect estão concluídas: oito combinações determinísticas (cinco quad-fault, duas penta-fault e uma hexa-fault).

## Estado atual - 2026-09-22 (Phase 186 reconnect after outage + jitter + duplicate receiver path)
- Teste headless compõe outage, jitter e duplicate determinísticos antes do reconnect, confirma playout pós-reconexão, classificação de duplicatas em `late_packets`, estado `Playing` e zero `output_failures`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 185 reconnect after outage + reorder + duplicate receiver path)
- Teste headless compõe outage, reorder e duplicate determinísticos antes do reconnect, confirma playout pós-reconexão, classificação de duplicatas em `late_packets`, estado `Playing` e zero `output_failures`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 184 reconnect after outage + loss + duplicate receiver path)
- Teste headless compõe outage, loss e duplicate determinísticos antes do reconnect, confirma playout pós-reconexão, classificação de duplicatas em `late_packets`, estado `Playing` e zero `output_failures`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 183 reconnect after outage + loss + reorder receiver path)
- Teste headless compõe outage, loss e reorder determinísticos antes do reconnect, confirma playout pós-reconexão, estado `Playing` e zero `output_failures`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 182 reconnect after outage + loss + jitter receiver path)
- Teste headless compõe outage, loss e jitter determinísticos antes do reconnect, confirma playout pós-reconexão, estado `Playing` e zero `output_failures`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 154 reconnect after bandwidth + loss receiver path)
- Teste headless compõe bandwidth e loss determinísticos antes do `OpusReceiver`, executa reconnect e confirma seis frames reproduzidos, um reconnect, estado `Playing`, três frames PLC e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 153 reconnect after bandwidth + jitter receiver path)
- Teste headless compõe bandwidth e jitter determinísticos antes do `OpusReceiver`, executa reconnect e confirma dez frames reproduzidos em ordem, um reconnect, estado `Playing`, zero PLC e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 152 reconnect after bandwidth + reorder)
- Teste headless compõe bandwidth e reorder determinísticos, executa reconnect e confirma dez frames reproduzidos, um reconnect, estado `Playing`, zero PLC e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 151 reconnect after bandwidth + outage)
- Teste headless compõe bandwidth e outage determinísticos, executa reconnect e confirma nove frames reproduzidos, um reconnect, estado `Playing`, zero PLC e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 150 reconnect after outage + reorder)
- Teste headless compõe outage e reorder determinísticos, executa reconnect e confirma 10 frames reproduzidos, dois frames PLC, estado `Playing`, um reconnect e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 149 reconnect after outage + jitter)
- Teste headless compõe outage e jitter determinísticos, executa reconnect e confirma 10 frames reproduzidos, três frames PLC, estado `Playing`, um reconnect e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 148 reconnect after outage + duplicate)
- Teste headless compõe outage e duplicação determinísticos, executa reconnect e confirma playout pós-reconexão, classificação de duplicatas em `late_packets`, estado `Playing` e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 146 outage + loss + duplicate receiver path)
- Teste headless compõe outage, loss e duplicate determinísticos antes do `OpusReceiver`, confirma 11 frames reproduzidos, cinco frames PLC, duas duplicatas em `late_packets`, estado `Playing` e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 144 reconnect after loss receiver path)
- Teste headless compõe perda determinística e reconnect, confirma seis pacotes reproduzidos, três frames PLC totais, mix recuperado, um reconnect, estado `Playing` e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 143 reconnect after jitter receiver path)
- Teste headless compõe jitter e reconnect, confirma metadados de recuperação de mix, perda de fronteira, sete frames reproduzidos e estado `Playing`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 130 bandwidth + duplicate receiver path)
- Teste headless compõe bandwidth e duplicação determinísticos, confirma seis pacotes únicos, três duplicatas classificadas como `late_packets`, zero PLC, estado `Playing` e zero falhas de saída. Evidência CODE local; WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e hardware permanecem pendentes.

## Estado atual - 2026-09-22 (Phase 129 bandwidth + outage receiver path)
- Teste headless compõe bandwidth e outage determinísticos, confirma sequência sobrevivente, dois frames PLC consecutivos, estado `Playing` e zero falhas de saída. Evidência CODE local; WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e hardware permanecem pendentes.

## Estado atual - 2026-09-21 (Phase 129 combined bandwidth/loss receiver path)
- Teste headless compõe `Stage::Bandwidth` e `Stage::Loss` sobre payloads Opus e valida PLC determinístico, playout, ausência de falhas e estado `Playing`. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual - 2026-09-21 (Phase 128 variable-payload bandwidth boundary)
- Teste unitário confirma que `BandwidthProfile` consome orçamento por bytes para payloads de tamanhos distintos, preserva ordem e descarta overflow deterministicamente. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

- Teste headless aplica `BandwidthProfile` a payloads Opus codificados e valida playout do receiver para pacotes admitidos, sem falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-22 (Phases 134-142 combined receiver fault coverage)
- `headless_receiver.rs` now covers outage+jitter, outage+loss, outage+duplicate, outage+reorder, jitter+reorder, jitter+duplicate, loss+reorder, loss+duplicate and reconnect-after-outage pipelines through `OpusReceiver`.
- Focused integration gate: 27 tests passed. Evidence is CODE local; real network, WebRTC/DTLS-SRTP, PipeWire/ALSA and Raspberry Pi hardware remain unvalidated.

## Estado atual - 2026-09-21 (Phase 126 combined bandwidth fault stage)
- `Stage::Bandwidth` integra `BandwidthProfile` ao `CombinedFaultProfile`; teste unitário cobre encadeamento bandwidth→loss. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual - 2026-09-21 (Phase 125 combined fault profile pipeline)
- `CombinedFaultProfile` e `Stage` enum adicionados ao crate `network-fault`: encadeia Loss/Reorder/Duplicate/Jitter/Outage sem dispatch dinâmico. Rejeita lista vazia com `FaultError::InvalidParameter`. Testes unitários cobrem estágio único e cadeias duplas; integração headless confirma 12 pacotes Opus através de Loss→Reorder→Duplicate com `packets_received=9`, `late_packets=3`, `plc_frames_total=2`. Evidência CODE+CI (PR #230 mergeada, 13/13 SUCCESS); runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual - 2026-09-21 (Phase 124 duplicate packet receiver path)
- `ReceiverError::DuplicateSequence` distingue pacotes duplicados de inválidos no `JitterBuffer::push`. `OpusReceiver::playout` chama `record_late()` para duplicatas e `record_dropped()` para inválidos. `DuplicateProfile` injeta duplicatas em intervalos regulares. Teste headless confirma classificação correta. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual - 2026-09-21 (Phase 123 deterministic receiver PLC burst limit enforcement)
- `OpusReceiver` agora tem cobertura para limite de budget PLC com outage de cinco pacotes consecutivos: quatro frames PLC validos, quinto frame ausente falha fechado, `output_failures` conta uma unica transicao e chamadas posteriores nao duplicam contador. Evidencia CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

# TODO

## 2026-09-24 — Batch Phase 363–372 streaming registry boundaries
- [x] Cobrir cap de drain oversized, FIFO/requeue, overflow, idempotência de remoção, replacement bounded e independência da fila de transporte; 230 testes `streaming` PASS localmente. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.


## 2026-09-24 — Phase 275 negotiated MediaBridge zero-output preservation
- [x] Cobrir `SessionRegistry::drive_once` com sessão negociada e `output_budget == 0`: bridge não consome frame antes de orçamento disponível; evidência CODE local.
- Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual — 2026-09-24 (Phase 274 MediaBridge recovery drop accounting)
- [x] Cobrir entrega após recuperação de fila: frame aceito depois de overflow não incrementa novamente contadores de descarte por sessão ou agregados. Evidência: teste focado PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual — 2026-09-23 (Phase 257 deterministic media session snapshots)
- [x] Ordenar snapshots de sessões por `user_id` para remover nondeterminismo de `HashMap`; teste de regressão PASS localmente. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual — 2026-09-23 (Phase 249 zero-budget media drain boundary)
- [x] Cobrir budget zero em `drain_session_frames_with_budget`: nenhuma mídia é removida e frame enfileirado permanece disponível no próximo drain.
- Evidência: teste focado PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-23 (Phase 243 Unicode whitespace media-session user ID boundary)
- [x] Cobrir IDs compostos apenas por whitespace Unicode em `MediaPlane::register_session`, sem mutação do registry.
- Evidência: teste focado `register_rejects_unicode_whitespace_only_user_id_without_mutation`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.


## Estado atual - 2026-09-23 (Phase 234 replacement credential boundaries)
- [x] Cobrir replacement credential exatamente em `MAX_CREDENTIAL_BYTES` e rejeitar tamanho excedente sem alterar dispositivo revogado.
- Evidência: testes unitários `replacement_credential_at_maximum_length_is_accepted` e `oversized_replacement_credential_is_rejected_without_mutation` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## Estado atual - 2026-09-22 (Phase 145 reconnect after bandwidth receiver path)
- Teste headless compõe admissão determinística de bandwidth e reconnect, confirma oito frames reproduzidos, um reconnect, estado `Playing`, zero PLC e zero falhas de saída. Evidência CODE local; rede real, WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.


## Linux-first release strategy — 2026-09-19
- [x] Define Debian/Ubuntu/Raspberry Pi OS support for amd64/arm64; Raspberry Pi 3 is family baseline.
- [x] Separate `SOFTWARE_RELEASE_GATE`, `PACKAGE_RELEASE_GATE`, `HARDWARE_CERTIFICATION`, `OPTIONAL`, `OBSOLETE`.
- [x] Add initial `.deb` builder, systemd unit, lifecycle scripts, validator and Make targets.
- [x] Complete real amd64/arm64 `.deb` artifact matrix and package install/upgrade/uninstall/purge in clean containers. amd64 lifecycle evidence: PASS on 2026-09-19; arm64 artifact and lifecycle evidence: PASS in CI run 35533396414 (CODE + PACKAGE_RELEASE_GATE).
- [ ] Complete software WebRTC/Opus E2E evidence. PipeWire/WirePlumber virtual sink/source enumeration and deterministic Opus writer/receiver round-trip are covered in CODE and CI; full WebRTC/Opus E2E remains pending.
- [x] Run configurable software stability profile with bounded test commands over deterministic CODE/SIMULATED audio and media paths; `run-software-release-gates.sh` now repeats headless audio plus streaming tests for `OPENIEM_SOAK_SECONDS`. This does not validate realtime, PipeWire, WebRTC runtime or hardware.
- [ ] Hardware certification remains separate: physical Pi 3/4/5, USB, hot-plug, latency, XRUN, thermal and power.


## Estado atual — 2026-09-21 (Phase 118 deterministic outage receiver path)
- Perfil determinístico `OutageProfile` agora dirige uma janela contígua de perda com dois frames PLC pelo `OpusReceiver` headless; cobertura confirma `plc_consecutive_max == 2`, saída válida e ausência de falhas. Evidência CODE local; WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 117 deterministic network fault receiver path)
- Perfil determinístico `LossProfile` agora dirige payloads Opus reais por `OpusReceiver` headless em teste bounded, cobrindo perda, PLC e métricas sem falha de saída. Evidência CODE local; WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 116 observability metrics reset)
- Endpoint `POST /api/v1/metrics/reset` protegido por Engineer/Admin limpa contadores de áudio, dispositivo, stream, receiver e rede sem alterar estado de áudio ou sessões. Engineer Console expõe `Resetar Contadores` com refresh após sucesso e falha fechada. Evidência CODE+CI; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 114 headless UDP/Opus loopback)
- Teste bounded de loopback UDP valida envio pelo `TransportAdapter`, encaminhamento manual ao `OpusReceiver` headless, 48 kHz estéreo/20 ms, PCM não silencioso e métricas sem falha. Evidência CODE local; não cobre negociação WebRTC completa, DTLS-SRTP, PipeWire/ALSA ou hardware.

## Estado atual — 2026-09-21 (Phase 111 decoder/output failure metrics)
- `ReceiverMetrics.output_failures` cobre falhas de decoder, PLC, frame PCM inválido, exaustão PLC e saída; testes unitários confirmam contagem única no latch fail-safe. Evidência CODE; integração headless, runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 110 late_packets counter)
- `late_packets` separado de `packets_dropped` em `ReceiverMetrics`: pacotes stale/duplicados incrementam `late_packets` via `record_late()`. REST `GET /api/v1/metrics` expõe campo. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 109 receiver fail-safe output metrics)
- `ReceiverMetrics` registra `output_failures` e `OpusReceiver` contabiliza transições de falha para mute fail-safe; cobertura de erro de saída, exaustão PLC e snapshot adicionada. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 108 receiver metrics round-trip coverage)
- Teste de integração do round-trip Opus confirma contadores compartilhados de recebido, descarte e reconnect. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 106 receiver metrics boundary)
- **Phase 106 review:** cobertura local confirma métricas de receiver para pacotes recebidos, overflow de jitter/ingress, reconnect e payload inválido; serialização REST confirma `schema_version` e contadores. Commits verificados: `2ac4a77`, `3ba17dc`, `48fa714`, `49e0d1d`. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

- **Phase 106 follow-up:** `OpusReceiver::enqueue` contabiliza payloads vazios ou acima de 1500 bytes como `packets_dropped`, com teste unitário para contagem única por rejeição. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

- **Phase 106:** métricas do receiver estão conectadas ao `OpusReceiver` e expostas no snapshot observability, mas nenhum binário headless consome `AppState.metrics.receiver`; integração de runtime permanece pendente. Não criar claim de WebRTC/DTLS-SRTP, PipeWire/ALSA ou Raspberry Pi 5.

## Estado atual — 2026-09-21 (Phase 105 receiver ingress drop metric)
- **Phase 105:** `OpusReceiver::enqueue` agora registra `packets_dropped` quando a fila bounded de ingress rejeita pacote por overflow; teste dedicado confirma contagem única. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 101 PLC concealment)
- **Phase 101:** `OpusReceiver` aplica concealment PLC bounded para gaps Opus, trava mute após quatro frames PLC de 20 ms e falha fechado em erro de decoder/output. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes. Frames Opus fora do contrato fixo de 20 ms também falham fechado, preservando limite temporal do PLC.

## Estado atual — 2026-09-21 (Phase 121 deterministic reorder receiver path)
- Revisão de `JitterProfile` corrigiu deslocamento de índices em múltiplos eventos de jitter e adicionou cobertura determinística. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.
- Perfil determinístico `ReorderProfile` agora entrega pacotes Opus reordenados ao `OpusReceiver` headless; cobertura confirma playout ordenado, zero PLC, zero pacotes tardios e ausência de falhas. Evidência CODE local; WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-21 (Phase 120 deterministic reconnect receiver path)
- **Phase 120:** `ReconnectProfile` agora tem cobertura integrada com `OpusReceiver`, incluindo perda na fronteira, recuperação de mix, contador de reconnect e playout pós-reconexão. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Estado atual — 2026-09-20 (P0-003 media readiness queue guard)
- **P0-003:** `SessionRegistry::drive_once` no longer drains MediaPlane frames until negotiated audio media exists, preventing frame loss during Sans-IO session setup. Regression coverage passes in CODE; runtime WebRTC/PipeWire/ALSA and hardware remain pending.

## Estado atual — 2026-09-20 (GAP-018/019 DTLS fingerprint binding)
- **GAP-018/019:** pairing aceita fingerprint DTLS-SRTP SHA-256 canônico opcional e ofertas autenticadas com fingerprint registrado exigem correspondência exata antes de criar sessão; identidades pareadas legadas sem fingerprint permanecem compatíveis. Evidência CODE local; runtime WebRTC/DTLS-SRTP e hardware permanecem pendentes.

## Estado atual — 2026-09-20 (GAP-018/019 PairingRegistry integration)
- **GAP-018/019:** `PairingRegistry` integrado ao api-server via PR #189 (CI 16/16 SUCCESS, SHA d80b7ef); rotas `POST /api/v1/audio/pairing`, `DELETE /api/v1/audio/pairing/:device_id` e autenticação de dispositivo no `/api/v1/audio/offer` implementadas com 7 testes de integração. DTLS-SRTP session binding e validação runtime/hardware permanecem pendentes.
- **GAP-018:** negociação de oferta agora rejeita identidade pareada revogada antes de criar sessão; evidência CODE local, DTLS-SRTP fingerprint binding e runtime permanecem pendentes.

## Estado atual — 2026-09-20 (Phase 99 — smoke PipeWire virtual no CI)
- **Phase 99:** o CI executa smoke software de PipeWire/WirePlumber com grafo virtual sink/source e round-trip Opus determinístico. Evidência CODE+CI/SIMULATED; runtime alvo, WebRTC em rede, latência e hardware permanecem pendentes.
- **Phase 100:** round-trip Opus agora cobre dois pacotes fora de ordem, reordenação por jitter buffer e conteúdo por frame; evidência local CODE. WebRTC em rede, PipeWire runtime e hardware continuam pendentes.

## Estado atual — 2026-09-18 (Phase 95 review)
- **Phase 95:** review canônica adicionada para `GET /api/v1/metrics`; endpoint permanece CODE/CI, sem validação de runtime ou hardware.

## Estado atual — 2026-09-18 (scene duplication status reconciliation)
- **P2 Scene duplication:** `POST /api/v1/scenes/{id}/duplicate` and Engineer Console action are merged with CODE+CI evidence; runtime and hardware validation remain pending.

## Estado anterior — 2026-09-18 (Device Manager status reconciliation)
- **P1-002 Device Manager:** capability discovery, bounded snapshot and recovery state machine plus protected `GET /api/v1/devices` are implemented and covered by CODE/CI; backend hot-plug/runtime and hardware validation remain pending.

## Estado anterior — 2026-09-17 (SceneStore numeric boundary hardening)
- **SceneStore active-revision consistency:** listagem, duplicação e save agora falham fechado para ponteiro zero, histórico ausente ou ponteiro acima do histórico. Evidência CODE local; runtime permanece pendente.

- **SceneStore numeric boundary hardening:** revisões SQLite negativas e overflow do contador agora falham fechado antes de produzir ou persistir estado inválido. Evidência CODE local; runtime permanece pendente.

## Estado anterior — 2026-09-17 (P2 locked-channel preset UI)

- **P2 Preset locked-channel UI:** Engineer Console identifica canais `locked`, impede seleção/aplicação e mostra estado bloqueado; cobertura CODE local. Runtime permanece pendente.

## Estado anterior — 2026-09-17 (P2 scene duplication)

- **P2 Scene duplication:** API `POST /api/v1/scenes/{id}/duplicate` e ação no Engineer Console implementadas; Engineer/Admin only, cópia inicia revisão 1 e não altera cena ativa. Evidência CODE+CI; runtime permanece pendente.

## Estado anterior — 2026-09-17 (P2 locked-channel preset boundary)

- **P2 Preset locked-channel boundary:** aplicação server-side rejeita canais `locked` antes de qualquer mutação; cobertura de integração valida resposta e preservação de estado. Evidência CODE local; CI/runtime permanecem pendentes.

## Estado anterior — 2026-09-17 (P2 preset catalog/application status reconciliation)

- **Architecture closure:** P0 technical decisions closed in ADR-001..010; implementation/validation follow `docs/DEVELOPMENT-HANDOFF.md`.
- **P1-008 — API/UI:** rotas de domínio `GET /api/v1/system` e `GET /api/v1/channels` implementadas na PR #75; EQ UI implementada na PR #74; cenas REST e catálogos Musician/Engineer concluídos em código/CI; aplicação de presets built-in no Engineer Console concluída nas PRs #144–#151. Runtime permanece pendente.
- **P1-002:** GET /api/v1/devices com RBAC, DeviceManager no AppState; mergeado em main (PR #80; CI 13/13).
- **P1-003:** Observability concluído (PR #68; CODE+CI/SIMULATED; runtime permanece pendente).
- **P1-004:** RecoveryRegistry integrado ao ciclo WebSocket de Musician; concluído em main via PR #81, CI run `35055191463` (13/13). CODE+CI; runtime/hardware permanecem pendentes.
- **P1-007:** backup/restore sem secrets implementado na PR #73; CLI local `iem config backup/restore` adicionada na Phase 96 com limite de 1 MiB e rejeição de symlink. Evidência CODE+CI/SIMULATED; restauração em ambiente limpo, API/uso operacional e runtime permanecem pendentes. P1-006 release permanece bloqueado por confirmação explícita e validação física.
- **Phase 83–95:** backend/control changes, musician channel names, observability metrics REST endpoint implementados e mesclados em `main`; runtime de áudio/media permanece não validado.
- **Release `v0.3.1`:** tag existe e CI remoto do HEAD passou 13/13; GitHub Release ainda não publicada por exigir confirmação explícita.
- **Validação headless/emulada:** DSP determinístico, Docker ALSA userspace e ALSA Loopback host passaram; válido para regressão/release headless, sem claim de hardware.
- **Validação física/runtime:** PipeWire/ALSA no target, WebRTC/Opus real, instalação Linux dedicada e Raspberry Pi 5 continuam pendentes.
- **Guias:** espelhos PT/EN/ES existentes; revisar e sincronizar após validação física.
- **P1-015 — Versioned SQLite migrations:** concluído em CODE; `M001` registrado em `migrations`, reaplicação evitada no reopen e schema legado compatível quando `must_change_password` já existe.
- **P0-007 first-access password:** login exposes `must_change_password`; authenticated `PUT /api/v1/auth/password` replaces Argon2id hash and clears bootstrap flag; ordinary users are denied. CODE evidence; runtime remains pending.
- **P0-003 media handoff:** PRs #119 and #125 merged; GAP-001 implementation is CODE/SIMULATED complete, while real WebRTC/network runtime validation remains pending. In-memory `TransportAdapter::send_from_registry` UDP delivery coverage now passes in CODE. `MediaBridge`, bounded `SessionRegistry::drive_once`, `MediaWriter` Opus encoding, negotiated `str0m::media::Writer` boundary and bounded `TransportAdapter` socket owner exist at CODE/SIMULATED; failed sends requeue within bounded capacity. Runtime and hardware remain pending.
- **P2 Engineer Console scenes UI:** listar, criar, recuperar, editar revisões e deletar cenas integrado às rotas REST em `web/engineer`; cobertura CODE permanece nos testes do console.
- **P2 Engineer Console scene revisions:** edição JSON e criação de nova revisão via PUT integradas; cobertura CODE+CI (PR #136, run `35189802675`).
- **P2 Musician scenes read-only:** catálogo autenticado de cenas e cena ativa integrado à Musician UI; músico não recebe permissão de recall/criação. Evidência CODE local; runtime permanece pendente.
- **P2 Musician scene catalog tests:** cobertura de renderização, estados loading/error/vazio e refresh adicionada; PR #140 mergeada com CI 13/13 (run `35194150143`). Evidência CODE+CI; runtime permanece pendente.
- **P2 Preset catalog/application single source:** catálogo e aplicação usam allowlist única server-side; criação, edição, persistência e presets de mix continuam fora do escopo. Evidência CODE local; runtime permanece pendente.
- **P2 Preset catalog/application:** `GET /api/v1/presets` fornece catálogo read-only para Musician/Engineer/Admin; `POST /api/v1/presets/{id}/apply` aplica presets built-in somente para Engineer/Admin em canal validado; Engineer Console oferece seleção de canal e aplicação autenticada em código/CI. Criação, edição, persistência e aplicação de presets de mix permanecem fora do escopo. Evidência CODE + CI (PRs #144–#151); execução e integração em runtime/hardware permanecem pendentes.
- **P2 Engineer preset validation:** entradas inválidas são descartadas no cliente; payload não-array cai em estado vazio sem quebrar renderização. Evidência CODE local; runtime permanece pendente.
- **P2 Engineer preset feedback:** aplicação exibe confirmação com preset/canal e mantém erro fail-closed; respostas obsoletas não alteram estado. Evidência CODE local; runtime permanece pendente.
- **P2 Preset application boundary:** `channel_index` fora do limite é rejeitado antes de mutação; payload JSON com campos desconhecidos falha fechado. Cobertura CODE local; runtime permanece pendente.
- **P2 Musician scene catalog refresh:** botão autenticado de atualização manual adicionado; respostas concorrentes e respostas após logout são descartadas. Cobertura CODE via typecheck/test/build; runtime permanece pendente.
- **P2 SceneStore revision invariant:** cenas restauradas agora exigem `revision >= 1`; cobertura unitária rejeita revisão zero antes de persistência. Evidência CODE; runtime/hardware permanecem pendentes.
- **P2 Scenes backup/restore:** PRs #94–#97 mergeadas; exportação durável, restore atômico, rollback de ponteiro ativo e cobertura RBAC concluídos em CODE+CI (run `35126301166`, 13/13). Restauração em ambiente limpo coberta por teste de reabertura file-backed; validação operacional implantada permanece pendente; seleção do caminho persistente agora é testável sem mutação global de ambiente.

## Phase 92 — Controle de EQ por WebSocket

- [x] Especificar `SetEqBand`/`EqBandAck` no control-protocol.
- [x] Validar band index, frequência, gain, Q e mix index no control-server.
- [x] Definir broadcast de `EqBandDelta` no api-server com RBAC para Musician.
- [x] Executar fmt, clippy, testes completos, revisão independente e scan de segurança.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e review da Phase 92.
- [x] Commitar e abrir PR #54 (mergeado em main via `chore/docs-cleanup-status-sync`).
- [x] Confirmar CI remoto 11/11 para mudanças mergeadas anteriores; cada novo commit exige evidência no HEAD.
- [x] Frontend EQ controls — Phase 93 implementada e mergeada em main via PR #74.
- [x] P0-001 — substituir Mutex no callback JACK por fronteira SPSC bounded e fila de controle non-blocking; callback processa período completo; testes locais passam.
- [x] P0-002 — Audio Lab L1/L2: harness SIMULATED com 8 testes CI implementado (PR #57).
- [x] P0-003 — Media Plane: bridge/session/Opus/negotiated writer plus bounded `TransportAdapter` socket owner implemented; bounded UDP delivery, failed-send requeue, dequeue-budget and suffix-order coverage pass in CODE/CI/SIMULATED. Runtime WebRTC/PipeWire/ALSA and hardware remain pending.
- [x] P0-004 — Native/headless Opus receiver core: bounded ingress/jitter, decode, fail-safe mute e reconnect (SIMULATED; OS output/hardware pendentes, PR #59).
- [x] P0-005 — Clock: sample timestamps, bounded drift estimator e adaptive resampling (SIMULATED; hardware clock validation pendente).
- [x] P0-006 — registry bounded de pairing, identidade, binding músico/mix, revogação, re-pair explícito e digest Argon2id salted; integração API, autenticação de ofertas e binding de fingerprint DTLS-SRTP concluídos em CODE+CI. Runtime/hardware permanecem pendentes.
- [x] P0-007 — bootstrap idempotente `soundtech` e fronteira de migração versionada (CODE+CI; PR #62).
- [x] P0-008 — ALSA explicit fallback backend: abertura PCM, hw_params, fail-safe mute, XRUN recovery, stop_flag Release/Acquire (CODE+CI; PR #63; PipeWire/hardware validation pendente).
- [x] P1-004 — Recovery: AppState RecoveryRegistry, lifecycle WS de conexão/desconexão, restauração de mix atribuído, guarda de ownership duplicado e conflito de assignment DB; 5 testes de recovery. PR #81; CI run `35055191463` (13/13); gates locais fmt, clippy, testes e documentação PASS. Runtime PipeWire/ALSA, WebRTC/Opus e hardware permanecem pendentes.
- [x] P1-005 — Network Tests: 5 perfis determinísticos (loss/jitter/reorder/outage/reconnect) no crate `network-fault`. Integração com `recovery` e `observability`. 47 testes verdes (CODE; PR #72 mergeado em main; CI remoto 12/12).
- [x] P1-008 — API/UI: `GET /api/v1/system` pública e `GET /api/v1/channels` protegida por RBAC, com testes de contrato; PR #75. Status de áudio permanece `SIMULATED`.


## P1 backlog status

- [x] P1-003 — observability concluído; runtime/hardware pendentes.
- [x] P1-004 — recovery concluído: PR #81, CI run `35055191463` 13/13, gates locais PASS; sem validação física.
- [x] P1-005 — network concluído (PR #72).
- [ ] P1-006 — release bloqueado: confirmação explícita, instalação real e Raspberry Pi 5 ainda pendentes.
- [x] P1-007 — backup/restore de configuração sem secrets implementado na PR #73; CLI local `iem config backup/restore` adicionada na Phase 96; 8 testes unitários; restauração em ambiente limpo e integração operacional ainda pendentes.
- [x] P1-008 — concluído (PR #75).
- [x] P2 — SceneManager SQLite: `SceneStore` implementado com revisions imutáveis, recall transacional e detecção de payload corrompido; PR #86, CI 13/13 (run `35067216669`). CODE+CI; runtime/hardware pendentes.
- [x] P2 — Scenes REST API: 7 rotas REST (GET/POST /api/v1/scenes, GET /api/v1/scenes/active, GET/PUT/DELETE /api/v1/scenes/{id}, POST /api/v1/scenes/{id}/recall) com RBAC, SceneStore integrado ao AppState; 7 testes de integração; PR #87, CI 13/13. CODE+CI; runtime/hardware pendentes.
- [x] P2 — Scenes REST API PUT coverage: revisão Engineer e bloqueio RBAC Musician cobertos por testes de integração.
- [x] P2 — SceneStore file-backed: `SCENE_STORE_PATH` seleciona SQLite persistente; teste cobre criação, reopen e leitura do payload. CODE+CI; backup/restore operacional e runtime pendentes.
- [x] P2 — Backup/restore operacional: `GET/PUT /api/v1/config/backup` com RBAC Engineer/Admin, payload estrito e integração ao `ControlState`; CODE+CI/SIMULATED. Restore em ambiente limpo e runtime permanecem pendentes.
- [x] P2 — SceneStore export/restore: `GET /api/v1/scenes/backup` exporta e `PUT` restaura cenas duráveis e ponteiro ativo em JSON versionado; validação estrita e substituição atômica; estado transitório excluído; CODE+CI/SIMULATED.
- [x] P2 — Scenes REST backup/restore API integration coverage: Engineer-only atomic replacement, active-pointer restore and Musician denial tests.
- [x] P2 — AppState SceneStore restart coverage: fresh `AppState` reopens same file-backed path and preserves scenes, revision, payload and active pointer; deployed runtime validation remains pending.
- [x] P2 — Preset catalog read-only: `GET /api/v1/presets` Engineer/Admin e catálogo no Engineer Console; aplicação/mutação permanece fora do escopo. CODE+CI; runtime/hardware pendentes.
- [x] P1-009 / Phase 94 — concluído; validação contra servidor/runtime/hardware permanece pendente.

## Phase 95 — Observability Metrics REST endpoint

- [x] Adicionar rota `GET /api/v1/metrics` protegida por RBAC Engineer/Admin.
- [x] Integrar `observability::AllMetrics` ao `AppState`; snapshot via `state.metrics.snapshot()`.
- [x] Implementar 3 testes de integração: role guard, schema_version=1, contadores inicializados a zero.
- [x] Executar fmt, clippy, testes (71 testes de integração), revisão independente e scan de segurança.
- [x] Commitar, abrir PR #78 e confirmar CI remoto 13/13 (run `35046851737`).
- [ ] Validar contadores contra áudio real e Raspberry Pi 5.

## Phase 94 — nomes de canais na Musician UI

- [x] Consumir `GET /api/v1/channels` com Bearer em memória.
- [x] Exibir nomes retornados pelo servidor, com fallback seguro para nomes padrão.
- [x] Validar payload, limites de índice e comprimento de nome.
- [x] Executar typecheck, 50 testes e build do frontend musician.
- [x] Commitar, abrir PR #77 e confirmar CI remoto 13/13 (run `35044749428`).
- [ ] Validar UI contra servidor real e Raspberry Pi 5.

## Phase 93 — Frontend EQ Controls (Engineer Console)

- [x] Adicionar componente `EqBandControl` no Engineer Console com sliders de frequência, gain e Q por banda.
- [x] Conectar ao WebSocket existente via mensagem `SetEqBand`.
- [x] Exibir `EqBandAck` e sincronizar estado com snapshot REST.
- [x] Executar typecheck, testes, build, revisão independente e scan de segurança.
- [x] Commitar, abrir PR e confirmar CI remoto.

## Phase 89 — Engineer Console: Channel Strip de gain/mute

- [x] Renderizar canais do snapshot `/api/v1/state` no Engineer Console.
- [x] Adicionar slider de gain `-144..+12 dB` e botão mute por canal.
- [x] Preservar Bearer auth, debounce, optimistic UI e guards contra respostas obsoletas.
- [x] 26 testes, typecheck e build aprovados; CI remoto 11/11 no PR #49.
- [x] PR #49 mesclado em `main`.
- [ ] Validar servidor real e Raspberry Pi 5.

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

## Phase 91 — validação de coeficientes Biquad contra referência

- [x] Adicionar vetores determinísticos de referência para quatro combinações de frequência, ganho e Q.
- [x] Validar coeficientes RBJ normalizados (`b0`, `b1`, `b2`, `a1`, `a2`) com tolerância de `5e-6` em `f32`.
- [x] Executar testes locais do crate `mix-engine`.
- [ ] Validar processamento de áudio em Raspberry Pi 5 real.


---

## Phase 87 — Engineer Console: controles master gain/mute via WebSocket

- [x] Adicionar `protocol.ts` com tipos WebSocket do engineer (`SetMasterGain`, `SetMasterMute`, `MasterAck`).
- [x] Implementar hook `useEngineerWs` com reconexão, guard de cleanup e handlers MasterAck/State/Error.
- [x] Integrar `WsBadge` e `MixMasterControl` (slider gain + botão mute) na App do engineer.
- [x] 18 testes aprovados, typecheck limpo, build limpo.
- [x] CI remoto 11/11 jobs aprovados (PR #47).
- [ ] Validar controles em servidor real e Raspberry Pi 5.

## Phase 84 — Reprodutibilidade do pipeline de release

- [x] Adicionar `--locked` a clippy, testes e builds Rust do workflow de release.
- [x] Normalizar timestamps, ordenação e ownership dos archives server/web com `SOURCE_DATE_EPOCH` do commit.
- [x] Adicionar gate que executa dois empacotamentos consecutivos e compara checksums em cada job de artefato.
- [ ] Confirmar dois builds do mesmo tag com checksums idênticos em runner CI.
- [ ] Publicar release `v0.3.1` após secrets, instalação real e validação final.
- [ ] Validar Raspberry Pi 5 real.

## Phase 83 — Pan e mudo master na UI do músico

- [x] Adicionar slider de pan por canal (-1 a +1) com rótulo L/C/R.
- [x] Desabilitar pan quando canal mudo.
- [x] Exibir badge MASTER MUTED quando servidor reporta mudo master (somente leitura).
- [x] Sincronizar pan do snapshot e enviar SetSendPan via WebSocket.
- [x] 42 testes, typecheck e build locais aprovados.
- [x] Implementar SetMasterMuted no protocolo quando suporte ao servidor estiver disponível.
- [x] Confirmar CI remoto verde no run `34761731828` (9 jobs aprovados após correção do pin de `actions/cache`).
- [ ] Validar Raspberry Pi 5 real.

## Phase 82 — revisão de segurança do instalador

- [x] Auditar fluxo de instalação após Phase 81.
- [x] Corrigir cópia repetida de UI que criava `dist/dist` ou preservava conteúdo obsoleto.
- [x] Exigir SHA completo, verificar checkout exato e recusar fonte mutável antes de build.
- [x] Publicar release versionado via staging limpo e restaurar `current` em falha de instalação.
- [x] Validar Node.js >= 20 antes de mutar host ou instalar runtime suportado.
- [x] Corrigir SHA inválido de `actions/cache` que quebrava todos os 10 jobs de CI (ff0c085).
- [x] Confirmar CI remoto verde nos runs `34761731828` e `34763037883` (9 jobs aprovados).
- [ ] Confirmar instalação real em host Linux dedicado.
- [ ] Desbloquear release e validar Raspberry Pi 5 real.

## Phase 81 — hardening do instalador e concorrência CI/release

- [x] Fazer `--dry-run` descrever todas operações sem alterar host.
- [x] Aceitar `--ref` como SHA-1 completo com checkout detached.
- [x] Validar caminhos usados em substituições `sed`.
- [x] Restaurar CI automático em `main` e serializar release por tag.
- [ ] Confirmar instalação real em host Linux dedicado.
- [ ] Desbloquear release e validar Raspberry Pi 5 real.

## Phase 80 — compatibilidade TypeScript 7 no Engineer Console

- [x] Adicionar `web/engineer/src/vite-env.d.ts` para declarar imports de assets Vite.
- [x] Validar typecheck, testes e build do Engineer localmente.
- [x] Confirmar CI remoto verde e atualizar PR #41.
- [ ] Validar release, instalação ARM64 e Raspberry Pi 5 real.

---

## BLOCKER

- [x] P0 — Confirmar correção da suíte Python de assinatura no GitHub Actions — runs `34761731828` e `34763037883` passaram nos 9 jobs.
- [ ] P0 — Publicar release `v0.3.1` após validar artefatos e instalação real.

---

## Phase 79 — compatibilidade OpenSSL 3.5 na geração de assinaturas

- [x] Adicionar `-rawin` aos helpers de teste Ed25519.
- [x] Validar suíte combinada localmente: 56 testes aprovados.
- [x] Confirmar CI remoto verde no run `34760297250` e atualizar documentação.
- [ ] Validar release, instalação ARM64 e Raspberry Pi 5 real.


---

## Phase 78 — rejeição de dados residuais em archives

- [x] Rejeitar bytes residuais e streams gzip concatenados após archive válido.
- [x] Adicionar testes offline para os dois casos.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 77 — validação estrutural antes do payload

- [x] Validar nomes, raiz, tipos, allowlist e duplicatas antes de consumir payloads.
- [x] Adicionar teste que garante rejeição de membro inesperado sem leitura de payload.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 76 — leitura integral de payloads de archive

- [x] Consumir payload completo de cada arquivo regular em chunks limitados.
- [x] Rejeitar payload truncado ou ausente durante leitura limitada pelo tamanho declarado.
- [x] Adicionar teste offline de payload truncado.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar archives e instalação em Raspberry Pi 5 real.

## Phase 278 — preservação de frames sob budget compartilhado por sessão

- [x] Cobrir duas sessões negociadas com budget de saída compartilhado em chamadas sucessivas.
- [x] Confirmar que frames roteados para ambas as sessões permanecem disponíveis após cada drenagem bounded.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar mídia WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 real.

## Phase 75 — fonte única de versão e gate reproduzível

- [x] Criar `VERSION` canônico e validar manifests Rust/frontend e tags SemVer.
- [x] Integrar gate ao workflow de release e aos testes locais.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e review.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar build/runtime ARM64, mídia WebRTC, PipeWire/ALSA e Raspberry Pi 5 real.

## Phase 74 — documentação visual das interfaces

- [x] Documentar interfaces reais implementadas em `docs/INTERFACES.md`.
- [x] Gerar SVG/PNG documentais sem secrets e marcar como mockups, não screenshots de runtime.
- [x] Atualizar README, CHANGELOG, DEVELOPMENT-LOG e guias afetados.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar UI em runtime, mídia WebRTC, PipeWire/ALSA e Raspberry Pi 5 real.

## Phase 73 — Guia do Músico alinhado ao signaling HTTP

- [x] Atualizar guia com sequência real de oferta SDP e trickle ICE via HTTP.
- [x] Marcar control plane/Sans-IO como validado e mídia como `SIMULATED`.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar mídia WebRTC, PipeWire/ALSA e runtime em Raspberry Pi 5 real.

## Phase 72 — integração HTTP de signaling

- [x] Cobrir oferta SDP autenticada e resposta via rota HTTP.
- [x] Cobrir trickle ICE autenticado após negociação na mesma sessão.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar mídia WebRTC, PipeWire/ALSA e runtime em Raspberry Pi 5 real.

## Phase 71 — snapshot validado para publicação

- [x] Fixar bundle de entrada por descritor de diretório e abrir membros com `O_NOFOLLOW`.
- [x] Copiar em chunks limitados para staging privado e publicar output somente após validação.
- [x] Limitar bundle a 32 entradas e adicionar testes de atomicidade, cleanup e leitura limitada.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 70 — hardening do validador de bundle

- [x] Limitar tamanho de arquivo, bundle e manifesto antes de consumir conteúdo.
- [x] Validar SBOM opcional como JSON objeto e rejeitar manifesto symlink.
- [x] Adicionar testes offline para SBOM inválido, limite de arquivo e manifesto symlink.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 69 — validação do bundle final de release

- [x] Validar conjunto final, versão, checksums, assinaturas server e arquivos inesperados antes da publicação.
- [x] Fixar fingerprint SHA-256 da chave pública e falhar fechado em ausência, formato inválido ou divergência.
- [x] Gerar manifesto de nomes/digests validados e publicar somente arquivos listados nele.
- [x] Integrar teste do validador de bundle ao job Python de segurança do CI.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Provisionar e distribuir chave pública Ed25519 por canal independente.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 68 — nomes Windows e limites de assinatura

- [x] Rejeitar caracteres inválidos, pontos/espaços finais e nomes reservados Windows em cada componente de archive.
- [x] Limitar assinatura e chave pública detached a 64 KiB antes de executar OpenSSL.
- [x] Adicionar testes offline determinísticos.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 67 — nomes seguros entre plataformas

- [x] Rejeitar separador `\\` e caracteres de controle em nomes de archive.
- [x] Adicionar testes offline para separador Windows e caractere de controle.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 66 — raiz canônica em archives

- [x] Rejeitar raiz `.`/`..` ou não canônica no validador.
- [x] Adicionar teste offline para raiz não canônica.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 65 — limites incrementais durante leitura de archives

- [x] Aplicar limites por membro e total descomprimido antes de consumir membros posteriores.
- [x] Adicionar teste offline de rejeição imediata para membro oversized.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 64 — tratamento fail-closed da portabilidade do validador

- [x] Converter ausência de `O_NOFOLLOW` em erro CLI controlado, sem traceback.
- [x] Validação local concluída.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 63 — verificação fail-closed de arquivos assinados

- [x] Exigir `O_NOFOLLOW` sem fallback permissivo, abrir com `O_NONBLOCK` e usar `/usr/bin/openssl` sem resolução por `PATH`.
- [x] Abrir artifact, assinatura e chave com `O_NOFOLLOW` e manter descritores estáveis durante OpenSSL.
- [x] Rejeitar symlink e entradas não regulares para todos os arquivos verificados.
- [x] Adicionar testes offline para symlink em artifact e assinatura.
- [x] Cobrir ausência de `O_NOFOLLOW` no validador de archives.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 62 — testes de segurança Python no CI

- [x] Adicionar job CI dedicado para testes dos validadores de archive e assinatura.
- [x] Fixar `actions/setup-python` por SHA completo.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 61 — abertura fail-closed de archives

- [x] Abrir archive com `O_NOFOLLOW` e medir tamanho no descritor validado.
- [x] Rejeitar entradas que não sejam arquivos regulares.
- [x] Adicionar testes offline para symlink e diretório.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 60 — validação incremental de archives

- [x] Interromper leitura ao exceder 32 membros, sem materializar headers além do limite.
- [x] Adicionar testes offline para limites comprimido e total descomprimido.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 59 — limites de recursos no validador de archives

- [x] Rejeitar archive acima de 512 MiB comprimidos, mais de 32 membros, membro acima de 256 MiB e total descomprimido acima de 512 MiB.
- [x] Adicionar testes offline para limites de contagem e tamanho.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

---

## Phase 58 — verificação local de assinatura Ed25519

- [x] Criar `scripts/verify-release-signature.py` com verificação detached Ed25519 fail-closed.
- [x] Adicionar testes offline para assinatura válida, artefato alterado, assinatura ausente e chave symlink.
- [x] Integrar geração/publicação de assinatura no workflow usando secret externo `OPENIEM_RELEASE_SIGNING_KEY_PEM`; chave nunca entra no repositório.
- [ ] Provisionar chave Ed25519 no GitHub Actions e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 57 — validação de archive no instalador e origem HTTPS

- [x] Usar `scripts/validate-release-archive.py` antes de extrair archive ARM64.
- [x] Definir `OPENIEM_ALLOWED_ORIGINS` no unit systemd e documentar ajuste para LAN IP.
- [ ] Desbloquear CI remoto antes de merge/release.
- [ ] Validar instalação em Raspberry Pi 5 real.

## Phase 56 — correção do lifetime do diretório temporário do Caddyfile

- [x] Manter `CERT_WORK_DIR` até concluir cópia e instalação do Caddyfile.
- [ ] Validar `caddy validate` e instalação em Raspberry Pi 5 real.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 55 — hardening de caminho no deployment

- [x] Resolver Caddyfile a partir da raiz confiável do clone.
- [x] Rejeitar Caddyfile symlink antes de instalação privilegiada.
- [ ] Validar `caddy validate` e instalação em Raspberry Pi 5 real.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 54 — provenance de artefatos de release

- [x] Gerar attestation de provenance para archives de servidor x86_64 e ARM64 antes do upload.
- [ ] Validar publicação e verificação da attestation após desbloquear GitHub Actions.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 53 — validação de archives de release

- [x] Validar archives de servidor x86_64/ARM64 antes de checksum/upload; rejeitar traversal, links, membros inesperados e binários ausentes.
- [x] Adicionar testes offline determinísticos, casos maliciosos e contrato CLI do validador.
- [x] Fazer instalador ARM64 exigir e instalar `api-server` e `open-iem-admin`.
- [x] Adotar assinatura independente para autenticar artefatos de release — integração Ed25519 no workflow concluída na Phase 58.
- [ ] Provisionar secret Ed25519 e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

## Phase 52 — follow-up de verificação

- [x] Impedir redirects no Admin CLI para preservar política HTTPS/loopback em cada requisição.
- [x] Fixar `cargo-audit` em `0.22.2` nos workflows CI e release.
- [x] Gerar certificados LAN em diretório temporário e instalar via `sudo install` com ownership/modos explícitos.
- [ ] Adicionar validação automatizada de archive malicioso e testar instalação em Raspberry Pi 5 real.
- [x] Adotar assinatura independente para autenticar artefatos de release — integração Ed25519 no workflow concluída na Phase 58.
- [ ] Provisionar secret Ed25519 e distribuir fingerprint/chave pública por canal independente.
- [ ] Desbloquear CI remoto antes de merge/release.

---

## Phase 51 — hardening do fluxo de deployment ARM64

- [x] Restringir redirects de download a HTTPS.
- [x] Criar diretórios de serviço com ownership e modos explícitos.
- [x] Gerar chaves JWT em diretório temporário e instalar com permissões restritas.
- [x] Resolver e validar unit systemd a partir de clone confiável antes da instalação.
- [ ] Validar instalação em Raspberry Pi 5 real após publicação de release.

## Phase 50 — hardening do instalador ARM64

- [x] Fazer download em diretório temporário com `set -euo pipefail` e limpeza automática.
- [x] Validar tag SemVer antes de construir URL/caminhos.
- [x] Rejeitar traversal, caminhos absolutos, symlinks e hard links antes da extração.
- [ ] Validar instalação em Raspberry Pi 5 real após publicação de release.

## Phase 49 — correção do guia de instalação ARM64

- [x] Alinhar nome e caminho do artefato Raspberry Pi ao workflow de release.
- [ ] Executar instalação em Raspberry Pi 5 real após publicação de release; runtime ARM64 continua não validado.

## Phase 48 — pinning imutável das actions

- [x] Fixar actions de terceiros de CI e release em commits SHA completos, com comentários de versão.
- [ ] Executar CI/release após desbloqueio do runner; não publicar artefato sem gates reais.

## Phase 47 — hardening dos workflows

- [x] Corrigir filtro de tags semver do workflow de release e reforçar validação exata no job `validate-version`.
- [x] Restringir permissões GitHub Actions; escrita limitada ao job `github-release`.
- [x] Pin de actions de terceiros por SHA completo; avaliação de atualizações permanece com Dependabot.
- [ ] Executar CI/release após desbloqueio do runner; não publicar artefato sem gates reais.

## Phase 46 — alinhamento do runner de release

- [x] Corrigir follow-up de revisão: documentar `make test-audio` e restringir validação PDF a `docs/guides`.
- [x] Trocar `ubuntu-24.04` por `ubuntu-latest` nos seis jobs de `.github/workflows/release.yml`.
- [ ] Executar workflow de release após desbloqueio do runner; validar gates, artefatos e checksums sem publicar artefato não testado.

## Phase 45 — correção do label de runner

- [x] Trocar `ubuntu-24.04` por `ubuntu-latest` em todos os jobs CI.
- [x] Confirmar resultado após push: run `34607886129` falhou antes dos steps em todos os jobs com `runner_id=0` e `steps=[]`.
- [ ] Desbloquear GitHub Actions/runner; runs `34614028392` e `34614025997` também falharam pré-steps (`runner_id=0`, `steps=[]`); API de permissões e runners retorna HTTP 403 para token atual; sem CI remoto verde não há release `v0.3.1`.

## Phase 43 — verification gates

- [x] Adicionar `server/Cargo.lock` e remover exclusão global do lockfile.
- [x] Adicionar job CI para documentação, PDF, skills e whitespace.
- [x] Renumerar ADRs duplicados 012 para 013/014.

## READY / CURRENT PHASE

- [x] P1 — Tornar `make up` reconstruível por padrão — Phase 40 — `docker compose up -d --build`; evita imagens obsoletas no desenvolvimento.
- [x] P1 — Validar Musician Guide PDF de forma reproduzível — Phase 42 — `make docs` extrai texto e renderiza PDF; `mutool` cobre ambiente sem `pdftotext`.
- [x] P1 — JWT pós-emissão revogável — Phase 31 — access-session mappings checked by middleware and WebSocket message/keepalive loops; local verification passes, CI remains blocked.

- [x] P1 — Corrigir Docker Compose dev — Phase 30 — Dockerfiles de desenvolvimento adicionados para API e UIs; `docker compose config` passa sem aviso de `version` obsoleto. Build completo depende de chaves JWT locais e daemon Docker disponível.
- [x] P1 — Implementar CLI `iem` com paridade documentada ao Makefile — Phase 35 — dispatcher tipado para help/status/diagnostics/docs/test/build/up/down; versão `0.3.1`; sem instalação global e sem `iem run`.
- [x] P1 — Corrigir saída do `open-iem-admin` — Phase 34 — HTTPS obrigatório fora de localhost, tabelas preservam união de colunas e 404 não afirma recurso planejado.
- [x] P1 — Criar harness de áudio determinístico — Phase 32 — cobre determinismo, sinais sintéticos, isolamento, ganho, pan, mute, limiter e finitude; não substitui hardware. Recuperação stop/start permanece follow-up.
- [x] P1 — Criar matriz formal de validação Docker/Linux/Raspberry Pi — Phase 37 — matriz publicada; execução em Raspberry Pi 5 continua pendente.
- [x] P1 — Adicionar sincronização de observabilidade ao desenvolvimento — Phase 36 — Engineer Console consulta telemetria e mantém métricas desconhecidas como `UNKNOWN`; áudio real segue pendente.

## BLOCKED / VALIDATION REQUIRED

- [ ] P0 — Validar PipeWire/ALSA e áudio real no Raspberry Pi 5 — status HARDWARE VALIDATION REQUIRED.
- [ ] P0 — Validar mídia WebRTC, latência, jitter, perda e recuperação — status HARDWARE VALIDATION REQUIRED.

---

## HIGH

- [x] Add API snapshot/telemetry contract before expanding Engineer Console (Phase 14; audio metrics remain SIMULATED)
- [ ] Install PipeWire on target hardware (Raspberry Pi 5) for Phase 1 validation
- [x] Evaluate audio transport options; WebRTC selected in ADR-004 (Phase 5 signaling scaffold complete)
- [x] Wire authenticated audio signaling routes and engineer session listing (Phase 5)
- [x] Establish authentication mechanism decision (ADR-008 — Accepted and implemented; TLS deployment validation pending)
- [x] HTTP integration tests for api-server (Phase 6 — 19 tests green)
- [x] Trickle ICE real injection in Sans-IO loop (Phase 6 — str0m Candidate::from_sdp_string)
- [x] WebSocket send mutations: SetSendGain/SetSendPan/SetSendMuted with musician ownership enforcement (Phase 16)
- [x] Biquad EQ real DSP (Phase 7 — TDF2 peaking biquad, RBJ coefficients, no heap)
- [x] RMS Compressor real DSP (Phase 7 — stereo-linked, exp-RMS, smoothed gain reduction)
- [x] Add ARM64 cross-compilation to CI (`rust-build-arm64` installs `gcc-aarch64-linux-gnu` and builds `aarch64-unknown-linux-gnu`; CI evidence: run `34792164989`)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)
- [x] Admin API server-side routes (Phase 9 — complete)

---

## MEDIUM

- [x] Initialize Rust workspace (`server/Cargo.toml`)
- [x] Initialize frontend projects (`web/musician/`, `web/engineer/`) — Phase 4 complete
- [x] Create Docker Compose for local development
- [x] Define PipeWire filter node architecture for Mix Engine (pipewire-jack — see docs/research/pipewire-integration.md)
- [x] Design channel/mix state machine (revision control — implemented in mix-engine)
- [x] Define WebSocket message type catalog (`server/control-protocol/`; Phase 3 foundation)

---

## LOW

- [x] Set up `cargo audit` in CI (Phase 8 — existing job)
- [x] Set up `npm audit` in CI (Phase 9 — npm-audit job added)
- [x] Create `examples/` with minimal mix scenario
- [x] Include deterministic `audio-engine` integration harness in `make test` — Phase 39
- [x] Configure Dependabot for dependency updates
- [x] Set up code coverage reporting
- [x] LOW: Log DB errors in master broadcast fan-out (fail-closed and observable)
- [x] LOW: mix_assignment_lock held during DB read in broadcast fan-out — lock removed from receiver-side fan-out; read-only ownership check now lock-free (Phase 86)
- [x] LOW: Test DB failure during musician WebSocket ownership lookup — fail-closed coverage added Phase 24
- [x] WebSocket keepalive: server Ping/Pong timeout and bounded socket sends — Phase 24 (policy timing covered; transport timing integration remains pending)
- [x] LOW: WebSocket keepalive: add deterministic state-machine coverage without waiting 30/60 seconds — tracker e loop cobrem timeout pendente, Pong incorreto, Pong correlacionado, ausência de falso timeout após Pong válido e fronteiras do intervalo 30 s/timeout 60 s
- [x] LOW: Aplicar limite de 16 KiB em mensagem/frame no `WebSocketUpgrade`, antes da alocação do payload — Phase 26 follow-up
- [x] LOW: Testar rejeição de frame/mensagem acima de 16 KiB via integração WebSocket — teste de transporte confirma encerramento para mensagem Text acima do limite (Phase 26 follow-up)
- [x] LOW: WebSocket: revogação pós-emissão de JWT — Phase 31; middleware, keepalive e mensagens revalidam access-session mapping persistido
- [x] LOW: WebSocket: quotas agregadas por usuário/IP — 4 por usuário, 16 por IP, 64 global; reserva atômica e liberação RAII (Phase 28)
- [x] HIGH: Limitar falhas de autenticação `/ws/v1` por IP — 5 falhas por janela de 60 s; estado bounded em memória, peer via `ConnectInfo<SocketAddr>` (Phase 29)
- [x] LOW: WebSocket: ressincronização após broadcast lag — servidor emite `State` com revisão autoritativa; Musician refaz snapshot REST (Phase 27); revisão monotônica cobre ACK antes do primeiro snapshot
- [x] LOW: WebSocket protocol control coverage: client Ping/Pong payload preservation and binary-frame rejection (Phase 24 follow-up)
- [x] LOW: WebSocket rate quota counts every inbound frame, including control frames, preventing Ping/Pong flood bypass (Phase 24 follow-up)
- [x] MEDIUM: Enforce process-wide WebSocket connection quota (64 permits); per-user/IP quotas and post-issuance JWT revocation checks implemented in Phases 28/31
- [x] LOW: Add explicit WebSocket logging redaction policy and generic client-facing protocol errors — Phase 25; parser/transport details redacted, normal close classified separately
- [x] LOW: Preserve validated WebSocket `request_id` in authorization errors — Phase 26; invalid envelopes use synthetic `server`

---

## RESEARCH

- [x] **Audio Transport** — Preliminary evaluation complete (docs/research/audio-transport/EVALUATION.md); WebRTC selected as primary; final benchmarks deferred to Phase 5
- [x] **Browser Audio Constraint** — Phase 90 verified WebRTC browser media versus native receiver constraints; real media validation remains pending
- [x] **PipeWire filter node API** — pipewire-jack selected for Phase 1 (docs/research/pipewire-integration.md)
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [ ] **Windows x64 audio backend** — Implement and validate WASAPI/ASIO backend; current server backend is not native Windows audio.
- [ ] **Platform validation matrix** — Keep Windows x64, Linux x64 and Raspberry Pi 5 ARM64 evidence separate; macOS, Android and iPadOS remain future/backlog.
- [x] **Linux realtime scheduling** — PipeWire+rtkit for Phase 1; hybrid in Phase 2 (docs/research/realtime-scheduling.md)

---

## PHASE 3 SECURITY FOLLOW-UP

- [x] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration — **DONE Phase 22** (Caddy config, systemd, RPi5 guide, ADR-011)
- [x] HIGH: Authorize every WebSocket message by role and musician mix ownership (Phase 23 — SetMasterGain/SetMasterMute RBAC complete; all WS messages now have explicit role checks)
- [x] HIGH: Enforce WebSocket expiry, rate limits and post-issuance revocation through persistent access-session mappings (Phase 31); revocation is checked on handshake, messages and keepalive ticks
- [x] HIGH: Make refresh rotation atomic in one SQLite transaction
- [x] MEDIUM: Add Origin/CSRF validation and bounded auth request inputs
- [x] MEDIUM: Bound HTTP request body to 16 KiB; route payload validation remains pending
- [x] MEDIUM: Parse bind address as SocketAddr and fail closed for non-loopback HTTP without explicit dev override
- [x] MEDIUM: Preflight refresh token owner and JWT before atomic rotation to avoid token loss on issuance failure
- [x] MEDIUM: Make musician mix ownership checks and send mutations atomic under `mix_assignment_lock` (Phase 15)

## PHASE 7 STATUS

- [x] Biquad peaking EQ real processing (TDF2, RBJ coefficients, stereo biquad state, 48kHz)
- [x] Stereo-linked RMS compressor (exp-RMS detector, smoothed gain reduction, mutators)
- [x] Docker Compose dev environment (api-server, musician-ui, engineer-ui)
- [x] Musician Guide pt-BR (docs/guides/MUSICIANS-GUIDE.md)
- [x] PHASE-6-REVIEW.md and PHASE-7-REVIEW.md
- [x] Integrate EQ + Compressor into Mix::process audio chain (Phase 8)
- [x] Create admin CLI for user management (Phase 8)

## PHASE 9 STATUS

- [x] Admin API server-side routes: GET/DELETE /api/v1/admin/users, GET/DELETE /api/v1/admin/sessions
- [x] DB methods: list_users, delete_user, list_active_sessions, revoke_session_by_id
- [x] ApiError::NotFound(String) variant
- [x] 8 HTTP integration tests for admin endpoints
- [x] 3 DB unit tests for new methods
- [x] Admin CLI fix: --username/--password for user create, --id for session revoke
- [x] Biquad coefficient validation vs Python/scipy (delta ≤ 5×10⁻⁸)
- [x] npm audit CI job (HIGH severity gate)
- [x] Admin self-delete protection (policy gate — deferred Phase 10)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)

## PHASE 8 STATUS

- [x] EQ + Compressor wired into Mix::process chain (Sum → EQ → Comp → Master Gain → Limiter)
- [x] Mix struct exposes `eq: ParametricEq` and `compressor: Compressor` fields
- [x] 5 new mix-engine integration tests (EQ boost, compressor reduction, passthrough, chain order)
- [x] Admin CLI binary (`server/admin-cli/`) — user/session/health commands, JSON/table output
- [x] Musician Guide PDF (`docs/guides/MUSICIANS-GUIDE.pdf`) — pandoc+xelatex, 61 KB
- [x] PHASE-8-REVIEW.md
- [x] Admin API server-side routes (Phase 9) — complete and covered by `server/api-server/src/routes/admin.rs` plus integration tests: GET/POST/DELETE `/api/v1/admin/users` and GET/DELETE `/api/v1/admin/sessions`.
- [x] Biquad coefficient validation vs reference implementation (scipy) — covered by deterministic reference vectors and tolerance tests; completed in Phase 91.
- [x] Generate Musician Guide PDF (Phase 8 documentation) — DONE Phase 8

## PHASE 1 STATUS

- [x] Audio engine crate with simulated backend and feature-gated JACK/PipeWire bridge
- [x] Phase 1 review completed: PASS WITH CONDITIONS
- [ ] Validate real PipeWire graph on a supported Linux host (`RUNTIME_VALIDATED`); physical Pi graph remains `HARDWARE_CERTIFICATION`.

## PHASE 21 STATUS

- [x] Replace WebSocket query-token authentication with negotiated subprotocol authentication
- [x] Reject missing, empty and query-only WebSocket credentials
- [x] Preserve server-selected subprotocol during HTTP upgrade
- [x] Update Musician client and integration coverage

## PHASE 20 STATUS

- [x] Add independent hook tests for REST snapshot, malformed nested state, delayed snapshot and `SendAck`
- [x] Validate WebSocket envelope version/request ID and ACK ranges client-side
- [x] Abort snapshot fetches when WebSocket connection is cleaned up
- [x] Replace WebSocket query-token authentication with cookie or subprotocol authentication — DONE Phase 21

## PHASE 18 STATUS

- [x] Musician client accepts `SendAck` revisions
- [x] Musician client ignores stale `State`/`SendAck` revisions
- [x] Add client reconciliation of full channel/send state after revision gap (Phase 19; REST snapshot + SendAck application)

## PHASE 17 STATUS

- [x] Broadcast WebSocket deltas for send gain/pan/mute
- [x] Engineer/Admin cross-session `SendAck` delivery
- [x] Musician mix ownership filtering for broadcasts
- [x] Assignment lock coordination and integration coverage
- [x] Independent security/code review; findings fixed
- [x] Add client-side revision ordering/reconciliation for missed or lagged broadcasts — Phase 27; stale revisions ignored and lagged snapshots reconciled

## COMPLETED

- [x] Repository initialized with correct GitHub remote
- [x] Environment audited and documented
- [x] Project directory structure created
- [x] 11 project skills created in `.agents/skills/`
- [x] Documentation structure created (`docs/`)
- [x] ADR baseline created (ADR-001 through ADR-008)
- [x] CI foundation created (GitHub Actions)
- [x] `scripts/validate-skills.sh` created
- [x] Root project files created (README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore)
- [x] Latency budget defined (`docs/audio/LATENCY-BUDGET.md`) — GAP-005 CLOSED
- [x] XRUN SLA defined (`docs/audio/AUDIO-SLA.md`) — GAP-006 CLOSED
- [x] Mix Engine Rust crate implemented (Channel, MixSend, Mix, Limiter, MixEngine)
- [x] 53 unit + doc tests passing — 0 clippy warnings

- [x] Lookahead brick-wall limiter (LOOKAHEAD_FRAMES=64 @ 48kHz = 1.33ms)
- [x] ParametricEq stub (passthrough — biquad DSP Phase 7)
- [x] Compressor stub (passthrough — dynamics DSP Phase 7)
- [x] 80 tests passing — 0 clippy warnings


## P1 TOPOLOGY STATUS

- [x] P1-001 — topology capability model and Channel Mode validation: `server/topology`, explicit source mapping, typed invalid-config errors, 11 tests (PR #65; CODE + CI local e remoto aprovados).
- [x] P1-002 — Device Manager: capability discovery, hot-plug and recovery state machine; integrado ao AppState, GET /api/v1/devices com RBAC; 3 testes de integração; CI remoto 13/13 (run `35052291469`; PR #80).

## PHASE 10 STATUS

- [x] SQLite mix assignment model and Engineer/Admin API
- [x] JWT numeric user identity (`uid`)
- [x] Musician-owned send gain/pan/mute routes
- [x] Ownership and assignment validation
- [x] Add dedicated Phase 10 integration coverage for assignment lifecycle and musician ownership routes


## PHASE 11 STATUS

- [x] Versioned release pipeline `.github/workflows/release.yml`
- [x] Version consistency gate (tag == workspace version)
- [x] Quality gate (fmt + clippy + tests + cargo-audit) required before builds
- [x] Linux x86_64 server artefact with SHA-256
- [x] Linux ARM64 cross-compile for Raspberry Pi 5 (SIMULATED — not hardware-validated)
- [x] Musician PWA + Engineer UI artefacts
- [x] GitHub Release with CHANGELOG excerpt
- [x] Workspace version bumped to 0.2.0; admin-cli aligned to workspace
- [ ] Tag v0.2.0 and verify full pipeline on GitHub Actions
- [x] Investigate `v0.3.0` release gate failure; tag points to pre-sync commit
- [ ] Publish `v0.3.1` and verify full pipeline on GitHub Actions — código local validado; CI remoto falha antes dos steps; logs bloqueados por permissão do token atual
- [ ] Validate ARM64 binary on real Raspberry Pi 5 hardware
- [x] Dependabot for Cargo + npm

## PHASE 12 STATUS

- [x] Fix release artifact naming (validate-version chain in build jobs)
- [x] Dependabot: Cargo + npm (musician/engineer) + GitHub Actions, weekly
- [x] Admin self-delete protection: 403 when caller deletes own UID
- [x] SBOM: cargo-sbom in release pipeline, best-effort non-blocking

- [x] SBOM generation (cargo-sbom in release pipeline)
- [x] Sincronizar versões com tag v0.3.0 após falha do gate de consistência
- [ ] Reapontar tag v0.3.0 e verificar pipeline completo no GitHub Actions

## Estado atual - 2026-09-23 (PairingRegistry identity ID boundary coverage)
- [x] Cobrir `device_id` e `musician_id` exatamente em 128 bytes e rejeitar 129 bytes sem mutar registry.
- Evidência: testes unitários `identity_ids_at_maximum_length_are_accepted` e `oversized_identity_ids_are_rejected_without_registry_change` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.

## 2026-09-24 — Phase 278 negotiated MediaBridge zero-frame-budget preservation

- [x] Cobrir `SessionRegistry::drive_once` com sessão negociada e `frame_budget == 0`: nenhum frame é drenado e chamada posterior entrega o frame preservado.
- Evidência: teste `drive_once_zero_frame_budget_preserves_negotiated_frames` PASS localmente; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e Raspberry Pi permanecem não validados.
