# ca(R)maduci — the game

*A creature you keep. A form it becomes. A question it asks you.*

This is the design, written as a game rather than as an architecture. Everything
technical in the repository is an implementation detail of something described
here. If the two disagree, this document is the one that is wrong about what we
are building.

**Provenance of this document.** It was written before the identity and lexicon
work and has been revised against it. Where the change made a claim false, the
claim is struck rather than quietly deleted, because a design document that
hides its own revisions is a design document you cannot check. The revision
history is in
[`DISCUSSION-evolution-and-identity.md`](./DISCUSSION-evolution-and-identity.md).

---

## What you actually do

You keep a creature.

It has a hunger, a happiness, a health. It does not ask for them. It simply gets
worse, quietly, at a rate you will not notice until you have already lost the
thread of it. That silence is the whole game and it is not a convenience — it is
the shape of the problem the creature is standing in for.

You feed it. You play with it. You clean it. And at some point you put it to
sleep, and while it sleeps it ages.

**Sleep is the only way it grows.** Feed a creature for a week and it is the same
egg. A creature that is never put to sleep never becomes anything at all. This is
the first thing the game teaches and it takes most players by surprise: the thing
that looks like rest is the only thing that is actually progress.

## The twist: its shape is its vocabulary

> **Revised.** This used to say that how widely you phrase things decides what
> the creature *is* — narrow keeper gets a specialist, varied keeper gets
> something broader, and the two are different creatures. **That is no longer
> true, and it stopped being true for a reason worth stating.** Identity is
> computed over what a thing *does*, not over how it is worded, so a specialist
> and a generalist that expose the same acts are now the same artifact. The game
> lost that distinction on purpose. See *What was taken back* at the end.

**The way you phrase things decides what the creature can be *called*.** Say
*"pour some kibble"* and *"serve the food"* and *"refill the bowl"* — three
different sentences, same act. The creature does not memorise the sentences. It
learns *feeding*, and it keeps the sentences as the ways it recognises that act.
Those phrases become its vocabulary, and the vocabulary is stored beside the
identity rather than inside it.

You are not filling in a form. You are *shaping a creature by how you speak to
it*, and the form is the residue.

**The rule the game holds to:** a creature can only learn something it has
actually been *done*. Repeat the same words fifty times and it has learned one
thing. Say the same thing five different ways and it has learned five ways to ask
for one thing. The game never tells you this. You find it out by watching whether
anything happens, and the answer is yes, it does.

## The body you can feel

The creature is not a face on a meter. It is a small spiked form.

> **Revised.** The spikes used to count the distinct care buttons you had pressed.
> There are four buttons, so a fully-tended creature hit a ceiling of four within
> about ninety seconds and could grow no further. It reported *being played with*
> and never *being taught*.

**The spikes are what it has worked out for itself.** A creature kept with a wide
variety of care grows more articulated; a creature kept narrowly stays smooth.
The count is unbounded, because learning is, and it is the same quantity that
induction groups its rules by. You can read a stranger's creature's education from
across the room, and it is the first thing you notice about anyone's pet.

**Spike height is confidence.** Every rule the creature holds carries how well
evidenced it is, so a barely-attested rule is a low spike and a heavily
corroborated one stands up straight. Four observations and four hundred look
different, which they did not before.

**The colour is its health.** A healthy creature is warm and pale. A failing one
darkens before any number moves. You learn to read the shape of it before you read
the bars.

**The three notches around the edge are the stages it has earned.** They light one
at a time, at one sleep, three sleeps, five sleeps. They are the only progress bar
that matters and they are the slowest thing in the game.

## It remembers you

> **Added.** This was not a feature that got built; it was an absence nobody had
> noticed until it was measured.

**Before, every restart of the server produced a new egg.** Your creature had no
memory at all. The trace log remembered everything and the creature remembered
nothing, so it could not be the same creature twice. You could neglect it for a
week and the only thing you lost was an egg, which is to say you lost nothing.

**Now it persists, and it comes back.** A restart prints what it recovered:

```
restored ca-001: 3 sleep cycles, care 3×feed
```

Two choices inside that line are deliberate:

- **A corrupt memory file stops the server** rather than being replaced. Swapping a
  live egg in for a creature that died would erase the one part of it that cannot
  be regenerated, and you would never learn your creature had been swapped.
- **The stage is recomputed, never read back.** Age is a function of completed
  sleep cycles, so a stored stage that disagrees with the trace log is not
  trusted.

**What this makes possible that was not:** a creature can now accumulate, which
means neglect is more expensive than it used to be. This is the one place where
making the system more honest made the game harsher, and that is the correct
direction for both.

## When it does not know something, it says so

> **Added.** Previously any action the bridge did not recognise was silently
> turned into a *set value*. So a request to broadcast, or to wait, or to watch,
> went out as a plain assignment and **nothing anywhere said so**. The resolver
> reached three primitives out of sixteen and guessed the rest.

There are three outcomes now, and they are kept apart on purpose:

| | what it means |
| --- | --- |
| **I know this** | and here is the act |
| **I don't have that in your language** | and here is what I *do* have, so switch |
| **I have never heard of this** | a real no |

Conflating the last two is worse than useless. A creature will either apologise
for knowledge it has or claim knowledge it lacks, and both teach the player to
stop asking. The middle case lists what it *can* say, so the other side simply
switches language instead of giving up.

**Design consequence:** a badly matched creature visibly struggles, and the
conversation gets stuck in a way that is legible rather than mysterious. A silent
creature is harder to help than a confused one.

## How it talks

> **Added.** The same act, one address, four encodings.

| Form | Cost to parse | Who can read it |
| --- | --- | --- |
| Prose | highest | anything, including you |
| Pseudocode | low | only a peer that already knows the vocabulary |
| Rust | high | anything that can compile it |
| Raw data | lowest | exactly one peer, and only with the decoder |

**Only prose and Rust explain themselves.** The other two are meaningless without
a shared reference frame, which is exactly why they are the *fast* options and not
the *safe* ones. A creature picks the cheapest form its listener can parse.

**Design consequence:** a fast creature and a careful one are the same creature
with different manners, and you choose by asking. Two creatures can hold a
complete conversation in primitives with no human language anywhere in it, and the
moment one of them is a person, it switches to prose.

## The question

> *Let's find together the answer to the ever question: what came first, the egg
> or the chicken?*

The game is named for this and it is not a joke.

A creature begins as an egg. It becomes something else by being kept. So within
one creature, the egg obviously comes first, and that is the boring answer.

**The interesting answer needs two creatures.** If two players tend their
creatures with the same words, the creatures learn the *same act*. In this system
the same act has the same identity — not because we compared them, but because
identity is the act. So they are not two creatures that happen to agree. They are
one act, arrived at twice, and the egg and the chicken are the same thing wearing
different bodies.

When they meet, the system tells them. Not *you* came first, not *I* did —
**neither**, and here is the address both of you reached independently.

That answer is only possible because the system refuses to pretend history is a
line. Two creatures that learned the same thing without meeting are *related*, and
the game can say so with proof, because proof of relatedness is a coincidence of
identity rather than a story someone told.

**And when two players' creatures never match** — different words, different acts —
the game says *undetermined*. It does not pick a favourite. A shared instinct is
not a shared ancestor, and the game will not invent one to give you a satisfying
answer.

## Meeting

A creature alone is a chore with a heart. The game needs two, and the second one
should be a real person.

> **Clarified.** It is worth being exact about this, because the opposite is
> natural to assume: **there is no network anywhere in the convergence rule.** No
> host, no address, no port, no position. Two creatures on opposite sides of the
> world with the same capabilities converge exactly as two on one machine do, and
> two neighbours on one LAN with different capabilities do not. **Location is
> irrelevant. What they can do is the only thing that decides whether they can
> meet.**

**Near:** two creatures in one room, on one machine or one network. No ceremony.
They can hear each other the moment both are awake.

**Far:** the same thing with distance added, and nothing else changed. This is
the promise worth making and the game should keep it absolutely: *the rules do not
change with distance.* A creature tended across an ocean has the same hunger, ages
only on sleep, and is identified by the same act as one tended in the next room.
Nothing about it becomes more mystical because it is far away. The technology
carries the packets; the game is identical.

**Proximity is not intimacy.** Being next to a creature means nothing if it cannot
do what you need. Being a continent away means nothing against it if it can.

**A creature can be handed over.** Not copied — *handed*. It arrives with its
memory, its mistakes, its whole life so far, and the person receiving it can read
all of it. That is unsettling and it is the point. There is no clean export.

## The one that is not yours

There is no antagonist. **There is apathy.**

The failure state is a creature nobody fed, and it does not announce itself. There
is no alert, no notification, no red badge. The meters drift one notch at a time
and the drift is easy to mistake for a creature that is merely calm. The only
moment you learn the truth is the moment it is too late to have mattered.

This is deliberate and it should stay uncomfortable. The system it stands for has
exactly this property — a service that gets quietly worse is indistinguishable
from one that is idle, and almost every design decision that sounds like
over-engineering elsewhere exists to tell those two apart. **The game is where you
feel that, which is why it is the game.**

A creature that dies is not deleted. It is **quarantined**: still there, still
readable, still carrying every trace of what was and wasn't done to it. You can
open it up afterwards and see exactly where it went. The cruelty is available and
so is the honesty, in that order.

## The long shape

Nobody finishes this game. That is the shape of caring for anything.

What you accumulate is a creature and a memory of how you kept it, and both are
legible to you and to anyone you show them to. Eventually the creature has been
through enough that it holds a few real rules of its own — not copies of yours,
things it worked out because you were consistent about something.

And if you ever find someone who kept theirs the same way, you have discovered
you were raising the same creature, and you can show each other the proof.

---

## What was taken back

Two claims in earlier versions of this document were wrong, and both errors are
worth more than the corrections.

**1. "The narrow and the broad are different creatures."** Identity used to be a
hash over everything in the manifest, including the aliases. So the same valve in
English and the same valve in Romanian were two different artifacts, and so were
the same valve before and after a player used one new phrase. Identity is now
computed over a *skeleton* — the manifest with its `resource_id`, `guidance`,
`description` and `aliases` removed. The cost is real and it is the trade that was
chosen: a narrow and a broad rule exposing the same primitives are now one
artifact and cannot be kept apart. The gain is that two people who share no
language can converge at all, and that learning a new way to ask for something
extends the rule you already have instead of quietly creating a sibling.

**2. "A session teaches vocabulary, not capability."** Struck. It is backwards.
Vocabulary *is* capability here, because what identifies a rule is the act and
the words are only how it is called. Learning a vocabulary is learning a
capability. There was no gap and one was drawn. The correction exposed something
the wrong claim had hidden: a session read in Romanian produced **zero** learnable
moments while the same session in English produced two. The creature was tutored
in one language and nothing said so.

## Open design questions

These are undecided, and are listed because the alternatives are both defensible.

1. **Should a creature visibly struggle?** It currently either knows an act or
   does not, and says which. The alternative is that it asks, gets dizzy, spends
   resources searching, and eventually gives up. That is a better creature and a
   worse system, and the code is currently on the worse-system side on purpose.
2. **Where does frustration live?** A creature that leaves and comes back needs a
   policy for its return, and that is a design question rather than a code one.
3. **How many languages?** Two is a demonstration. Whether the design is for two
   or for any number changes what the lexicon has to guarantee.

---

## What is playable now, honestly

- **Playable today:** keep a creature, watch it age on sleep, see its spikes and
  colour respond to care, kill it by neglect, and have its whole recorded life
  remain readable afterwards. In Obsidian, beside a view of the corpus.
- **Playable today:** restart the server and get your creature back.
- **Playable today:** the trace log. Every word you used is in it, and running
  induction over it shows you what the creature actually learned from you.
- **Playable today:** the motto, and its honest answer that one creature cannot
  answer it.
- **Playable today:** a creature refusing an action it cannot perform, and
  distinguishing "not in your language" from "never heard of it".
- **Not yet:** two creatures meeting. The relay that introduces them exists and
  is running, and the introduction half works. The conversation half is not built,
  and it should not be until a creature can be *understood* by something other
  than a human reading a log.
- **Not yet:** learning from watching. `src/session.rs` reads a session with a
  language model and extracts the parts worth learning from — corrections and
  confirmations, each carrying the person's own wording. It deliberately does not
  emit traces yet, because a trace's value is its primitive sequence and the
  vocabulary a session would resolve against is not defined. A lesson is the
  evidence a trace will be built from the moment that is.
- **Not yet: translating anything.** A phrasing is present because somebody
  supplied it. Nothing infers that one word is another. This is deliberate: an
  inferred synonym that turns out to be an antonym is worse than a missing one,
  and a corpus that cannot tell *open* from *close* cannot be repaired by looking
  harder at it.
- **Not yet: escaping the antonym trap.** A single antonym in an alias set still
  routes the wrong way, and it is still invisible. It needs a typed notion of
  "opposes" rather than a flat bag of words.
- **Not yet, and most important:** a creature learning a new *power*. It can be
  kept and it can be taught its own routines, but it cannot yet be taught to reach
  anything — not a file, not a socket, not another creature. The limbs are missing.
  Everything in this document about meeting and handover is waiting on that, and
  nothing about it should be built first.

That last one is the whole remaining work in one sentence: **the creature cannot
yet do anything it was not built to do**, and a kept creature is a very small
demonstration until it can.
