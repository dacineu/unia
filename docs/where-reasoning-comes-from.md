# Where reasoning actually comes from

A research note, and a correction to
[`creature-as-intermediary.md`](./creature-as-intermediary.md).

Written after reading Karpathy's nanochat work and the RLVR literature, both of
which say something this project had been assuming rather than checking. Two of
the assumptions were wrong, and one of them was wrong in the project's favour
last turn.

---

## 1. What "add an ability" means, according to the primary source

Karpathy's guide to adding an arbitrary computing capability to a model is
[nanochat discussion #164][164], "counting r in strawberry (and how to add
abilities generally)" (24 Oct 2025). He taught a d32 model to count letters in a
word. The method has four moving parts, and three of them are recognisable here.

**Synthetic task generation is the whole method.** A generator emits
conversations that challenge the model to do the thing, each with the intended
solution attached, mixed into midtraining and SFT. The model imitates. There is
no architectural change and no new architecture to invent.

**Entropy comes from varying the prompt, not the answer.** `USER_MSG_TEMPLATES`
holds many phrasings of the same question — and, per the post, *a few other
languages at the bottom so that nanochat gets triggered into this task even if
the user asks in a different language*. He is explicit that this is data
augmentation, and that it is what makes the ability fire however it is asked.

This is the [`Lexicon`](./SPEC.md) doing a job that a real system already needed
it to do. A phrasing is present because somebody supplied it, and supplying it in
several languages is what makes the act reachable from any of them.

**Decomposition is what makes a hard task learnable.** The post's summary of why
counting letters is hard: tokenisation is awkward, the model must break
multi-character tokens into individual characters, and it must compare-and-
increment as it goes. His fix is to spread those sub-computations across the
output so that no single forward pass does much:

> If you force a state of the art LLM to respond to this type of query in just a
> single token (no reasoning/thinking), then it has to solve all of the above in a
> single forward pass of the neural network, which is really, really hard.
> Reasoning models break up the task over many tokens, simplifying each forward
> pass substantially. It can then attend to all that intermediate work and treat
> it almost like little memory registers of partial solution.

And on making the steps small:

> smaller models care a bit more, and by obsessively scrutinizing the
> tokenization, you can make tasks a lot easier for them. And because smaller
> models aren't as capable, you also gain performance by really breaking up the
> entire problem into all the implicit calculations it implies.

**The staged recipe is "prior, then RL".** He is candid that the reasoning
structure in the synthetic data is, at first, smoke and mirrors — every instance
is clean, so the two paths always agree, and all he has done is set up a schema
and habituate the network to it. Two ways to make it real: simulate mistakes and
recoveries in SFT, or run RL on the task. And then:

> You show examples of reasoning behavior in SFT (we call this the "prior") and
> then once they are present and evoked with some probability, we expect RL to be
> able to take over and actually find a way to string it all together.

This is the same staged shape reported elsewhere — long-CoT cold start followed
by large-scale RL, as in the R1-lineage and LongCat recipes surveyed in
[Lambert's chapter on reasoning and inference-time scaling][rlhf]. Cold start
demonstrates the shape; RL makes it reliable.

## 2. The part that overturns what I said last turn

I told you that the creature's competence is bounded by the completeness of the
declarations, and presented that as a ceiling. The literature says something
sharper, and it is about the *mechanism*, not just the limit.

[RLVR does not create reasoning capability.][neurips] The NeurIPS 2025 analysis
finds that reasoning gains under RLVR are **bounded by the base model**: six
popular RLVR algorithms perform similarly and are far from fully exploiting what
the base already contains. And directly on point:

> we find that distillation can introduce new reasoning patterns from the teacher
> and genuinely expand the model's reasoning capabilities. Taken together, these
> findings suggest that current RLVR methods have not fully realized the potential
> of RL to elicit genuinely novel reasoning abilities in LLMs.

A second result says the same thing from the other side: most RLVR gains
decompose as **search compression** — `pass@k` to `pass@1` efficiency — with
capability expansion the minority. RL makes a model *more reliable*, not *more
capable*.

**So the teacher is not scaffolding to be removed. It is the only mechanism
observed to add capability.** "Independence" cannot mean "the teacher went away";
it has to mean *the teacher internalised its sequences*. A creature that has
distilled a sequence can then serve it forever without the teacher, and the
escalation rate measures how much of the teacher's repertoire has moved across.

This reorders the plan. The external transduction stage is not an accelerator to
be swapped for the internal matcher once things work. It is the supply of new
capability, and the internal matcher is what takes over afterwards.

## 3. The part that is a warning, not a mechanism

[Spurious rewards][spurious] and the associated re-analysis: on MATH-500,
Qwen2.5-Math-7B improved **21.4% with random rewards** against **29.1% with
ground-truth rewards**. Most of the "signal" was not signal. If the verifier is
weak, the system learns to satisfy the verifier.

unia's verifier is the declared semantics — the primitive exists, the state
transition matches `target_state`, the capability check passes. That is a better
verifier than GSM8K's final-integer check, because the environment is declared
rather than inferred.

**And one part of it is not implemented.** `constraints` is an array of free
strings that is printed and never evaluated. That is divergence D5, recorded
since early in the project as "the second pillar of the safety argument".

It is not that. Under this reading it is **the reward function**. A verifier that
does not check the declared preconditions is partly a random reward, and the
literature above is a demonstration of what that buys you. D5 stops being a
safety feature to be argued for and becomes the load-bearing assumption of the
entire learning loop. Everything the creature learns is a function of the
verifier, so **D5 is the first thing to fix, ahead of any model work.**

## 4. A hazard from the nanochat thread, applied here

In discussion #164 a user reported the model failing on a case, and Karpathy
diagnosed it as a stale checkout on his inference box: the Python tool was
returning a wrong answer while the model's own reasoning was right. His remark
on the fix:

> nanochat seems to trust the Python tool more when the two disagree

That is the general shape of the failure, and it is the shape unia is
deliberately built to invite. The verifier is authoritative; the reasoner defers
to it. That is the correct design — and it means **a wrong verifier is worse
than a wrong reasoner**, because the reasoner cannot overrule it. The properties
P1 to P4 in the plan are the containment boundary, and D5 is what sits on the
other side of it.

Two further failures in the same thread are worth recording because they are
predictable here. Counting *digits* failed because the training data was words
only — an out-of-distribution input. And a gibberish string was silently
normalised to a real word, so the model counted the wrong thing entirely. Both
are what an undeclared input does to a system with no way to say "I do not have
this". unia's answer to both is the contradiction gate, and the reason it must
fire on declared facts is that an inference cannot tell them apart.

## 5. What this changes in the plan

**D5 is promoted to first.** Evaluate `constraints`, or do not run a learning
loop. Under a weak verifier the loop optimises the wrong thing, and it will look
like it is working.

**The corpus is training data, and it is Stage 0.** The missing corpus — twelve
artifacts with zero shared capabilities — is simultaneously what blocks the
escalation rate *and* what the SFT stage has nothing to train on. Karpathy's
method is a task generator; unia's analogue is a **manifest generator** that
emits artifacts with declared state spaces and overlapping primitives. One piece
of work unblocks the cold-start stage, the escalation measurement, and the
convergence machinery, which are three of the project's open items. It is not a
footnote and it should not be scheduled last.

**Small vocabulary is a feature for reasoning, and only a limit for coverage.**
Last turn I led with "the vocabulary is 16, that is the ceiling". That conflated
two axes. For *reliability of each step*, 16 primitives is the enabling
structure — it is what makes each step small enough to be dependable, which is
precisely the decomposition argument above. For *open-domain coverage* it is a
hard limit that more artifacts relocate to whoever writes the declarations. Both
are true; they are different claims, and the reasoning argument depends on the
first rather than the second.

**Mutation must change behaviour or change nothing.** `MutationEngine::mutate`
appends a marker to `guidance` and adds a `provenance` block, which changes the
content address while leaving actions and state space byte-identical. Under the
spurious-rewards result that is the worst available outcome: it is a training
signal that rewards changing identity without changing what a thing does. If
mutation cannot alter behaviour, it must not alter the address.

**The verifier needs a name and a number.** "Does the creature's declared
completeness exceed X" is measurable today and is the quantity RLVR is bounded
by. It is a better diagnostic than escalation rate, because it says what the
system *cannot* learn rather than what it currently does not.

## 6. What is still not known

- **Whether the distillation transfers to artifacts rather than weights.** Every
  result above is about models. Distilling a behaviour into a content-addressed
  artifact and then serving it without the teacher has, as far as this note can
  establish, no published analogue that I could find. That gap is the research
  contribution, and it is also the largest risk in the plan.
- **Whether escalation rate will be worth reporting.** Twelve artifacts cannot
  produce an impressive number and a generated corpus may not produce a
  *meaningful* one. The metric's validity is unestablished.
- **How the declarations get written.** Every result above assumes they exist and
  are correct. Generating a manifest is easy; generating one whose `state_space`
  is complete enough that the verifier means something is the actual hard part,
  and nothing in this note helps with it.

[164]: https://github.com/karpathy/nanochat/discussions/164
[rlhf]: https://rlhfbook.com/c/07-reasoning
[neurips]: https://neurips.cc/virtual/2025/poster/119944
[spurious]: https://www.promptfoo.dev/blog/rlvr-explained
