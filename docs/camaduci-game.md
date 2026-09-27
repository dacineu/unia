# ca(R)maduci — the game

*A creature you keep. A form it becomes. A question it asks you.*

This is the design, written as a game rather than as an architecture. Everything
technical in the repository is an implementation detail of something described
here. If the two disagree, this document is the one that is wrong about what we
are building.

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

This is the mechanic the whole project rests on, and it needs no explanation to
the player because it is visible on screen.

**The way you phrase things decides what the creature becomes.**

Say *"pour some kibble"* and *"serve the food"* and *"refill the bowl"* — three
different sentences, same act. The creature does not memorise the sentences. It
learns *feeding*, and it keeps the sentences as the ways it recognises that act.
Those phrases become how it can be called.

Feed it the same way every single day, and it becomes a specialist: narrow,
reliable, very good at exactly one thing. Say it a hundred different ways and it
becomes something broader, because it has had to recognise the act across more
phrasing than any one of them.

You are not filling in a form. You are *shaping a creature by how you speak to
it*, and the form is the residue.

**The rule the game holds to:** a creature can only learn something it has
actually been *done*. Repeat the same words fifty times and it has learned one
thing. Say the same thing five different ways and it has learned five things. The
game never tells you this. You find it out by watching whether anything happens,
and the answer is yes, it does.

## The body you can feel

The creature is not a face on a meter. It is a small spiked form.

**The spikes are what it has been asked to do.** A creature kept through a wide
variety of care grows more spikes and a more articulated silhouette. A creature
kept narrowly has three and a smooth outline. You can see, from across the room,
how richly a stranger's creature has been engaged with. It is the first thing you
notice about anyone else's pet.

**The colour is its health.** A healthy creature is warm and pale. A failing one
darkens before any number moves. You learn to read the shape of it before you read
the bars.

**The three notches around the edge are the stages it has earned.** They light one
at a time, at one sleep, three sleeps, five sleeps. They are the only progress bar
that matters and they are the slowest thing in the game.

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

**Near:** two creatures in one room, on one machine or one network. No ceremony.
They can hear each other the moment both are awake.

**Far:** the same thing with distance added, and nothing else changed. This is the
promise worth making and the game should keep it absolutely: *the rules do not
change with distance.* A creature tended across an ocean has the same hunger, ages
only on sleep, and is identified by the same act as one tended in the next room.
Nothing about it becomes more mystical because it is far away. The technology
carries the packets; the game is identical.

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

## What is playable now, honestly

- **Playable today:** keep a creature, watch it age on sleep, see its spikes and
  colour respond to care, kill it by neglect, and have its whole recorded life
  remain readable afterwards. In Obsidian, beside a view of the corpus.
- **Playable today:** the trace log. Every word you used is in it, and running
  induction over it shows you what the creature actually learned from you.
- **Playable today:** the motto, and its honest answer that one creature cannot
  answer it.
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
- **Not yet, and most important:** a creature learning a new *power*. It can be
  kept and it can be taught its own routines, but it cannot yet be taught to reach
  anything — not a file, not a socket, not another creature. The limbs are missing.
  Everything in this document about meeting and handover is waiting on that, and
  nothing about it should be built first.

That last one is the whole remaining work in one sentence: **the creature cannot
yet do anything it was not built to do**, and a kept creature is a very small
demonstration until it can.
