# Paradise — Self-Evolving Code Intelligence

An AI coding agent that learns your style, remembers your patterns, and improves itself over time. Powered by the EDEN-ARBOR auto-evolutive architecture.

## Why Paradise?

| Feature | Copilot / Cursor | OpenCode | Paradise |
|---------|------------------|----------|----------|
| Learns your coding style | ❌ | ❌ | ✅ StyleLearner |
| Remembers across sessions | ❌ | ❌ | ✅ EWC + replay buffer |
| Self-improves its own code | ❌ | ❌ | ✅ ARBOR evolution engine |
| Multi-paradigm reasoning | ❌ | ❌ | ✅ FLUX/SPARK/ROOTS/WYRM |
| Works offline | ❌ | ✅ | ✅ |
| MIT open source | ❌ | ✅ | ✅ |

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/Sakumiyaz/ParadiseApp/main/install.sh | bash
```

## Quick Start

```bash
# Open the TUI (interactive terminal)
paradise

# Chat with ARBOR routing
paradise chat "refactor the auth module"

# WYRM reasoning without LLM
paradise think "fn main() { let x = vec.unwrap(); }"

# Run genome evolution
paradise agent arbor evolve

# Show learning stats
paradise stats
```

## Architecture

```
paradise (single binary)
├── TUI — glassmorphism interface, vim modes, command lens
├── CLI — 30+ commands
├── ARBOR — 24 modules
│   ├── FLUX — sub-quadratic spiking attention (O(L))
│   ├── SPARK — LIF spiking neural network with Hebbian learning
│   ├── ROOTS — hierarchical memory (neural + persistent + episodic)
│   ├── WYRM — scale-free Hebbian reasoning graph
│   ├── Ouroboros — deep recursive self-modification
│   ├── ParadigmBus — cross-paradigm signal channel
│   ├── Continual — EWC anti-forgetting + replay + surprisal
│   ├── WorldModel — internal simulation
│   ├── Planner — multi-step planning with replanning
│   ├── Metacognition — Aware→Reflective→SelfModifying
│   ├── Collective — multi-agent specialization
│   ├── SelfCode — sandboxed source code patches
│   ├── CognitiveLoop — perceive→reason→act→learn
│   └── LLMProvider — OpenAI/Anthropic/Ollama/OpenRouter
└── GARM — 123+ nodes, 91 capabilities, real tools
```

## Themes

- **Eclipse** — dark glassmorphism (default)
- **Dawn** — light glassmorphism

Toggle with `/theme` in the TUI.

## License

MIT