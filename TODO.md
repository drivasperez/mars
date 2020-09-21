# Todo List

## Bugs

### Parser

- [ ] Multiple Org pseudo-instructions should be accepted, and the last one used.
- [ ] END pseudo-instruction should equivalent to an ORG instruction.

### Core

- [x] What is the main number type? i32, u32, isize, usize? (usize for now)

## Features

### Parser

- [ ] Allow using global core variables in redcode, CORE_SIZE, MAX_INSTRUCTIONS etc.
- [ ] When parsing instructions with no given modifier:

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
- [ ] Random address and separation
- [ ] ~Fold_read and fold_write need to take isize, I think.~
- [ ] Change which task queue we're on from pointer to a queue, placing the queue back on the task queue queue if it has tasks, dropping if not. Keep track of which warrior each task queue corresponds to.

### Logging

- [ ] Improve DebugLogger to collect ongoing stats or whatever.
