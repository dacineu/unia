# What changed, from the creature's side

A companion to `camaduci-game.md`. That document is the game. This one is the
diff since it was written, described the way a player or a designer would notice
it, so the changes can be agreed with or argued against before they harden.

**Nothing below is cosmetic.** Every item changes either what a creature can do
or what it can tell you. Where a change made something worse, that is said.

---

## 1. The creature can now be *found*, and that changed its silhouette

**What you see:** spikes no longer mean "which buttons you pressed." They mean
**how many things it has worked out for itself.**

Before, a creature that had been fed, played with, cleaned and put to sleep had
four spikes and could grow no further — the count was the number of distinct
buttons, and there are four buttons. Its most expressive channel was full after
about ninety seconds. It reported *being played with*, never *being taught*.

Now the spikes are what it has learned, which is **unbounded**. A creature kept
narrowly stays smooth. A creature kept with variety grows more articulated. You
can read a stranger's creature's education from across the room, and it is the
same number induction groups by.

**Spike height is confidence.** Every rule carries how well evidenced it is, so a
barely-attested rule is a low spike and a heavily-corroborated one stands up. Four
observations and four hundred now look different, which they did not before.

---

## 2. The creature now remembers, which is the whole difference

**Before:** every restart of the server produced a **new egg**. Your pet had no
memory whatsoever — the trace log remembered everything, and the creature
remembered nothing. It could not be the same creature twice.

**Now:** it persists, and comes back. A restart prints what it recovered:

```
restored ca-001: 3 sleep cycles, care 3×feed
```

Two deliberate choices in that:

- **A corrupt memory file stops the server** rather than being replaced. Swapping
  a live egg in for a pet that died would erase the one part of it that cannot be
  regenerated, and you would never learn your creature had been swapped.
- **The stage is recomputed, never read back.** Age is a function of completed
  sleep cycles, so a stored stage that disagrees is not trusted.

**What this makes possible that wasn't:** a creature can now accumulate. Which
means neglect is more expensive than it was — a week of absence used to cost you
nothing but an egg, and now it costs you everything the creature had learned.

---

## 3. It is no longer monolingual, and the surface is the creature's, not the corpus's

**The change underneath:** what a creature *is* is no longer computed from how it
is *phrased*. Before, every alias went into the content hash. Measured, the same
valve in English and in Romanian were two different artifacts — and so were the
same valve before and after one player used a new phrase.

**What that fixes:** two creatures kept by two people who share no language now
converge. Two people who word it differently converge. And learning a new way to
ask for something extends the rule you already have instead of quietly creating a
sibling.

**The cost, stated plainly:** a narrow rule and a broad rule that expose the same
primitives are now **one artifact** and cannot be kept apart. Breadth of phrasing
is a property of how a thing is recognised, not of what it is. If you wanted
"the strict version" and "the loose version" as separate entries, that is gone.

---

## 4. A creature can now be asked things it does not know, and it says so

This is the change with the most design weight.

**A creature no longer guesses.** Previously any action it did not recognise was
silently turned into a *set value*. So a request to broadcast, or to wait, or to
watch, was dispatched as a plain assignment and **nothing anywhere said so**.

Now there are three outcomes, kept apart on purpose:

| | what it means |
|---|---|
| **I know this** | and here is the act |
| **I don't have it in your language** | and here is what I *do* have — so you can switch |
| **I have never heard of this** | a real no |

Conflating the last two is worse than useless: a creature will either apologise
for knowledge it has or claim knowledge it lacks. The middle case lists what it
*can* say so the other side simply switches language instead of giving up.

**Design consequence:** a creature that is badly matched will visibly say so, and
the conversation gets stuck in a way that is legible rather than mysterious. A
silent creature is harder to help than a confused one.

---

## 5. Convergence is not about location — it already wasn't

Worth stating because it's counter-intuitive and it's already true in the code.

**There is no network anywhere in the convergence rule.** No host, no address, no
port, no position. Two creatures on opposite sides of the world with the same
capabilities converge exactly as two on one machine do. And two neighbours on one
LAN with different capabilities do not.

**Location is irrelevant. What they can do is the only thing that decides whether
they can meet.**

The design consequence: *proximity is not intimacy.* Being next to a creature
means nothing if it can't do what you need. Being a continent away means nothing
against it if it can.

---

## 6. A creature can answer in four ways, and chooses by audience

The same act, one address, four encodings:

| Form | Cost | Who can read it |
|---|---|---|
| Prose | highest | anything, including you |
| Pseudocode | low | only a peer that already knows the vocabulary |
| Rust | high | anything that can compile it |
| Raw data | lowest | exactly one peer, and only with the decoder |

**Only prose and Rust explain themselves.** The other two are meaningless without
a shared reference frame — which is exactly why they are the *fast* options and
not the *safe* ones. A creature picks the cheapest form its listener can parse.

**Design consequence:** a fast creature and a careful one are the same creature
with different manners, and you choose by asking. Two creatures can hold a
complete conversation in primitives with **no human language anywhere in it** —
and the moment one of them is a person, it switches to prose.

---

## 7. The vocabulary is no longer the bottleneck — and the game is now honest about it

The line I wrote before was wrong and it's worth correcting visibly:

> ~~A session teaches vocabulary, not capability.~~

**Wrong.** Vocabulary *is* capability here. A rule that recognises "pour some
kibble" and one that recognises "toarnă porumb" are the same rule, because what
identifies it is the act and the words are only how it's called. Learning a
vocabulary is learning a capability. There was no gap and I drew one.

**What the correction exposes:** a session read in Romanian produced **zero**
learnable moments, and the same session in English produced two. The creature was
tutored only in one language, and nothing said so. That's fixed in the direction
you described — vocabulary becomes transductible, and the language model is a
transduction layer rather than a hardcoded list — but it is not done.

---

## What a player can and cannot do now

**Can:**

- keep a creature that survives restarts and accumulates what it learns
- read its education off its silhouette, with confidence encoded as height
- see it refuse an action it cannot perform, and say in which way it cannot
- hold a cross-lingual conversation between two creatures
- choose the encoding: prose, primitives, Rust, or opaque data

**Cannot, yet:**

- **teach it a new power.** It can be kept and it can learn its own routines, but
  it cannot be taught to reach anything it was not built to reach. The limbs are
  missing. This is the whole remaining work and it is what the network questions
  are waiting on.
- **translate anything.** A phrasing is present because somebody supplied it.
  Nothing infers that one word is another. This is deliberate: an inferred
  synonym that turns out to be an antonym is worse than a missing one, and a
  corpus that cannot tell *open* from *close* cannot be repaired by looking harder
  at it.
- **make a second player easily.** The relay that introduces two creatures exists
  and is running. The conversation half of it is not built.
- **escape the antonym trap.** A single antonym in an alias set still routes the
  wrong way, and it is still invisible. That needs a typed notion of "opposes"
  rather than a flat bag of words.

---

## Questions for you

1. **The identity reversal (§3).** Is losing the narrow/broad distinction right?
   The alternative is keeping phrasing in the hash, which costs cross-lingual
   convergence entirely. I'd defend the trade, but it was made without you.
2. **Should a creature visibly struggle?** Right now it either knows an act or
   doesn't. Your earlier idea was that it *asks*, gets dizzy, spends resources
   searching and eventually gives up. That is a better creature and a worse
   system, and the current code is on the "worse system" side deliberately.
3. **Where does frustration live?** A creature that leaves and comes back needs a
   policy for when it returns. That is a design question, not a code one, and it
   should be yours.
4. **How many languages?** Two is a demonstration. Is the design for two, or for
   any number? The answer changes what the lexicon has to guarantee.
