# ca(R)maduci — overview

**Status:** a runnable core exists as `cargo run --example camaduci`. The care
loop, the declared state space, the sleep-gated stages, the trace log and the
lifecycle transition are all implemented and tested. Rendering, assets, audio, a
score, and any notion of a player are not.
**Mark:** `ca(R)maduci` is a **placeholder**. See [Trademark](#trademark) before
using it in anything public.

---

## What it is

A digital pet in the Tamagotchi line: a small creature that must be cared for,
that evolves according to how it was treated, and that dies of neglect. The
difference from the 1996 original is the axis it evolves along.

`unia` already has the vocabulary for this. A pet is an **actuator** whose state
space is its vitals and whose action primitives are the care operations. Feeding
is `SetValue(hunger)`. Ignoring it is the absence of a call. The pet is not a
special case bolted onto the architecture — it is the smallest thing that
exercises the whole loop, which is why it is worth building.

| Tamagotchi concept | unia construct |
|---|---|
| The creature | a `.ure` actuator: `state_space` = vitals, `action_primitives` = care ops |
| Hunger / happiness / health | declared state variables with types and ranges |
| Feeding, cleaning, medicine | `UniversalPrimitive` actions on that actuator |
| A missed call | an escalation: nobody served the intent, so a provider is called |
| Growth stage | a lifecycle value, in the sense `crate::gc` already uses |
| What it becomes | the induced artifact's alias set and readiness |

That last row is the point. In a Tamagotchi, care choices pick a branch of a
hand-written evolution matrix. Here, care choices are **traces**, and the
creature's future form is whatever `crate::induce` derives from them. The pet
becomes a worked example of the paper's claim that an artifact is induced from
recorded interaction rather than authored.

## Research: what already exists

The space is not empty and the prior art is worth respecting.

**Hardware.** Bandai's Tamagotchi (1996) established the form: a 32×16 dot-matrix
LCD, three buttons, and a creature that dies if ignored. Bandai has since kept
the line alive with colour LCDs, zoom, and device-to-device communication
(Tamagotchi Paradise). Playmates' Nano Pets and Bandai's Digimon (1997) are the
main variants; Digimon's addition is the one that matters here — two units
**link and interact**, so a pet is not purely local state.

**Open source.** Several projects are directly relevant:

- **TamaFi** (ESP32-S3, MIT, ~400★) — WiFi-aware virtual pet. Mood is driven by
  the network environment: `CURIOUS` when hidden networks are nearby, `BORED`
  after prolonged disconnection, `SICK` on neglect. Stages run
  `BABY → TEEN → ADULT → ELDER` on wall-clock age. This is the closest existing
  implementation and the one to read first.
- **tama96** (Rust, MCP server + TUI + Tauri) — a desktop pet exposed as an MCP
  server with `feed`, `play_game`, `discipline`, `give_medicine`, `clean_poop`,
  `toggle_lights`, `get_status`. Written in Rust and speaking MCP, so it is
  architecturally the closest neighbour to this project.
- **M5Stack virtual-pet ecosystem** — a local, offline, privacy-first pet family
  on ESP32 hardware.
- **pseudoPet** (Python/Pygame), **MLubocki/ESP32-Virtual-Pet** (C++/OLED).

**One mechanic worth stealing.** tama96 documents that wake-time age only
advances once the pet has slept, and that a pet which never sleeps never reaches
the next stage. Age is gated on a completed cycle rather than accumulating on a
timer. That is a better model than a clock, and it maps directly onto `unia`:
progress is gated on a *completed interaction*, not on elapsed time.

## What exists today

`examples/camaduci.rs` runs the care loop and is covered by 15 tests. It compares
two creatures over the same number of ticks: one tended, reaching adulthood with
a trace log of `feed×12, play×8, sleep×6`; one neglected, staying an egg, dying,
and moving to `quarantined` rather than being deleted.

The example deliberately does not attempt to be a game. There is no renderer, no
input handling, and no score. What it establishes is that the loop closes: an
intent becomes a primitive, the primitive moves declared state, the interaction
becomes a trace, and the traces are what induction would read. The neglected
pet produces no traces at all, which is the same condition the induction loop is
currently blocked on.

## Design

### State space

Four declared variables, typed, so the constraint machinery has something to
check:

```
hunger     float  0.0..1.0     1.0 is starving
happiness  float  0.0..1.0
health     float  0.0..1.0
age_ticks  uint   monotonic, gated on sleep
```

### Action primitives

Care operations, each a `UniversalPrimitive` against this actuator. Neglect is
not an action — it is the absence of one, which is what makes it a real
obligation rather than a punishment the game can levy at will.

### Evolution

Three rules, in priority order:

1. **The age gate.** A stage advances only after a completed sleep cycle. Never
   sleeps, never advances. Borrowed from tama96.
2. **Care is recorded, not scored.** Every interaction is a trace. The creature's
   later form is induced from the trace log, not chosen from a table. This is the
   part that makes it a demonstration rather than a game.
3. **Death is a lifecycle transition, not a deletion.** The artifact moves to
   `Quarantined` in `crate::gc` terms and is retained. A pet that died is still
   inspectable, and the traces that killed it are still evidence.

### What it demonstrates, and what it does not

It would exercise: the actuator loop end to end, trace recording, induction from
real data, lifecycle transitions, and collection. `unia_record` currently has no
caller, and induction has never seen real usage — a pet that must be fed is the
cheapest way to produce genuine traces on demand.

It would **not** demonstrate: escalation rate at any interesting corpus size,
cross-model execution, or convergence between nodes. A single pet is one artifact
with no peer to converge with.

## Trademark

**`ca(R)maduci` is not a registered mark.** The `(R)` symbol is used throughout
this project on the understanding that it marks a mark the author intends to
register, and `TRADEMARK.md` records the status as *not cleared, not filed*. The
register is the authoritative record; the symbol elsewhere is a statement of
intent, and the two are kept consistent rather than merged.

If the name is ever challenged, what matters is the filing, not the symbol.
`TRADEMARK.md` already argues that MIT grants no protection to names, so a mark
is worth exactly nothing until it is registered, and the outstanding step is the
filing itself. 15 U.S.C. §1125 makes false designation of goods as registered a
civil cause of action in the US, with comparable EU and UK provisions, so the
filing is worth completing before the name is used commercially rather than
after.

The name should also be checked for availability. `TRADEMARK.md` lists the marks
already claimed in this project — `unia`, `unia-OS`, `UPA`, `DU-UUID`, `Mattern`,
`.ure` — and a clearance search is cheap now and expensive later.

## Open questions

- **Is this a game or a test fixture?** It is currently both, and those pull in
  different directions. A fixture should be deterministic and clock-injectable; a
  game should be fun. Deciding this first avoids building something that is
  neither.
- **Should the pet be an `.ure` artifact or a Rust struct?** An `.ure` keeps the
  state space declarative and lets the existing loader and matcher handle it. A
  struct is easier to test. The `.ure` route is the one that makes the pet a
  demonstration.
- **Does it need its own repository?** Probably. It is a game, it will want its
  own assets and its own release cycle, and unia is a research prototype. It can
  depend on unia as a library.
- **Multiplayer?** Digimon's linked interaction and the mesh architecture suggest
  two pets meeting. That is the most interesting version and the most scope.
