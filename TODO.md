# Todo List

## Bugs

### Parser

- [x] Multiple Org pseudo-instructions should be accepted, and the last one used.
- [ ] END pseudo-instruction should equivalent to an ORG instruction.

### Core

- [x] What is the main number type? i32, u32, isize, usize? (usize for now)

## Features

### Parser

- [ ] Allow using global core variables in redcode, CORE_SIZE, MAX_INSTRUCTIONS etc.
- [x] When parsing instructions with no given modifier:

> Oh, one more thing. How did I know which modifier to add to which instruction? (and, more importantly, how does the MARS add them if we leave them off?) Well, you can usually do it with a bit of common sense, but the '94 standard does defines a set of rules for that purpose.
>
> DAT, NOP
> Always .F, but it's ignored.
> MOV, SEQ, SNE, CMP
> If A-mode is immediate, .AB,
> if B-mode is immediate and A-mode isn't, .B,
> if neither mode is immediate, .I.
> ADD, SUB, MUL, DIV, MOD
> If A-mode is immediate, .AB,
> if B-mode is immediate and A-mode isn't, .B,
> if neither mode is immediate, .F.
> SLT, LDP, STP
> If A-mode is immediate, .AB,
> if it isn't, (always!) .B.
> JMP, JMZ, JMN, DJN, SPL
> Always .B (but it's ignored for JMP and SPL).

### Core

- [x] Helpers for running core, ie wrapping addition/subtraction, following indirect addressing and inc/decrementing ptrs.
- [x] Actually process instructions!
- [x] Report wins/losses
- [x] Random separation
- [ ] Random address
- [x] Change which task queue we're on from pointer to a queue, placing the queue back on the task queue queue if it has tasks, dropping if not. Keep track of which warrior each task queue corresponds to.
- [ ] Profile perf. Suspect using modulus operator (%) is slower than just boundschecking with if.

### Logging

- [ ] Improve DebugLogger to collect ongoing stats or whatever.

### Visualiser

- [ ] More UI (Show paused/unpaused, list of alive warriors etc)
- [ ] Switch to crossterm (termion is unix-only)
- [ ] First and second line of visualiser seem to be drawing in the same place.
- [ ] Back/forwards
- [ ] Reorganise code, reduce use of unwrap.
- [ ] Run visualiser on the main thread, no point spawning three threads and having main thread doing nothing.
