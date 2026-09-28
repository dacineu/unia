# Performance: what one act costs, and what it does not mean

Measured on the development machine, `--release`, stdout to a pipe, N = 50,000
acts. Reproduce with `cargo run --release --example dispatch_bench > /dev/null`.
Three runs; the spread is under 5% on every row.

## The retraction

This file exists because the benchmark it replaces was not a measurement.

`tests/benchmark_tests.rs` timed **one** dispatch against
`thread::sleep(800ms) + thread::sleep(50ms)`, printed the quotient as an
"Efficiency Gain", and asserted `gain > 1.0`. The numerator was a duration the
test slept through. It could not fail, and its output was a number with nothing
measured behind it. `docs/PROVENANCE.md` cited it as "empirical benchmarks" for
the first public commit. That citation is retracted above.

The comparison was **removed, not repaired**. Repairing it means asserting that
one unia act and one LLM inference are comparable quantities. That is a claim
about value, not about time, and it is not one a stopwatch can settle. A
fabricated baseline is worse than an absent one, because it prints a figure that
reads as a result.

## The measurement

| | ns/act | share |
| --- | ---: | ---: |
| whole act | ~14,300 | 100% |
| — `map_intent`: intent → action | ~11,000 | **77%** |
| — `dispatch`: gate, route, construct, execute | ~3,300 | 23% |
| — of which two `println!`s | ~2,100 | 15% |
| act, logging excluded | ~12,200 | |

Throughput: **~69,600 acts/s**.

## Scale reference: Lua 5.4.9, same machine

**This is not an equivalent.** It is the cost of work a Lua program also pays, so
the magnitudes are comparable and the *work* is not.

| | ns/op |
| --- | ---: |
| table field write | 6 |
| `string.format` + arithmetic | 275 |
| combined | 281 |

So: an act costs **~51× a combined Lua table write and format**, and the
dispatch half of an act costs **~550× a table write**.

## What that ratio does and does not mean

It does not mean "unia is 51× slower than Lua". One unia act is not one Lua
statement, and the arithmetic in that sentence is a category error dressed as a
benchmark. An act does five things a Lua statement does not:

1. resolve a natural-language intent to a declared action,
2. check the action's declared preconditions against reported state,
3. charge an economy,
4. route to every driver bound to the slot,
5. return a receipt describing what happened on the world.

**The 51× is the price of that list, and the list is the product.** A Lua program
pays none of it because it has none of it. If the comparison a reader wants is
"what does the *interpreting* cost, over and above the semantics", the honest
answer is the 3,300 ns dispatch row — and even that row is not a clean
interpreter, because it includes the gate and the charge.

## The finding that matters more than the ratio

**77% of an act is `map_intent`, not interpretation.** The cost is dominated by
turning the string `"Emergency shutdown the valve"` into the key
`emergency_shutdown`.

Looking at why:

- `intent.to_lowercase()` allocates, per call.
- For each action, `action.id.to_lowercase().replace('_', " ")` allocates again.
- On a non-matching action, `SemanticMapper::compute_score` builds two
  `HashSet<String>` of tokens and takes a set intersection and a union.
- `create_packet` then clones the params map, the constraints vec, the target
  state string, and allocates a request id.

So the hot path is repeated allocation and re-tokenisation of a string that did
not change. None of that is inherent to interpreting sixteen verbs.

Which means: **calling this an interpreter and asking how it compares to Lua
compares the wrong part.** The interpreter is the cheap 23%. The expensive 77% is
a semantic matcher, and a semantic matcher is exactly the thing `src/lexicon.rs`
and `src/induce/` exist for. The obvious next measurement is whether an induced
matcher replaces the per-call tokenisation, and it has not been made.

## What would make the comparison real

Three conditions, none of which currently hold:

1. **An agreed unit of work.** "One act" is only comparable to something once
   both sides are doing the same thing. Deciding that is a specification, and it
   is the same class of decision as the target-spec fork in
   `docs/unia-target-spec.md` §6.
2. **A fixture authored by someone who has not read the matcher.** The same
   constraint that blocks the 200-query fixture, and for the same reason: a
   fixture written by the author of the matcher measures the author's
   assumptions about intent.
3. **The 16 verbs doing work a Lua VM does.** Per §2.3 of the target spec, a
   `Signature` is a closed sequence with no call, no return and no locals, so
   there is no program to run and no loop to time. Until a program can be
   expressed, "unia vs Lua" has no program-shaped meaning.

## Reproducing

```sh
cargo run --release --example dispatch_bench > /dev/null   # unia
lua5.4 examples/lua_scale.lua                            # scale reference
```

The redirect matters: two `println!`s sit inside the timed loop, and they are
15% of the act. The benchmark measures them and subtracts them rather than
silently removing them, because they are in the shipped path.
