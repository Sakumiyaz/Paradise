# ADR-014: GARM Hybrid Voice Architecture

## Status

Accepted.

## Context

GARM has optional local TTS as a manifest-producing organ, but it does not include a neural voice model. A useful next step is to define a hybrid architecture that can later host a local stacked-transformer TTS backbone while preserving GARM's hierarchical, auditable control loop.

## Decision

Add a GARM-native hybrid voice seam with commands `hybrid voice`, `hybrid voice plan TEXT`, `hybrid voice synth TEXT` and `hybrid voice audit`. The seam records an architecture plan combining `stacked_transformer_backbone` with `garm_hierarchical_loop:text->intent->prosody->acoustic->verify`. `hybrid voice synth` writes `hybrid_voice_manifest.txt` and delegates to the existing optional TTS manifest flow.

## Consequences

- EDEN now has an auditable hybrid voice architecture target.
- No neural weights, GPU dependency, shell execution, network access or separate runtime are introduced.
- The existing TTS path remains optional and non-blocking.
- Future local neural TTS backends can consume the manifest without changing GARM's command surface.
