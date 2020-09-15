# Todo List

## Bugs

### Parser

- [ ] Multiple Org pseudo-instructions should be accepted, and the last one used.
- [ ] END pseudo-instruction should equivalent to an ORG instruction.

### Core

- [ ] What is the main number type? i32, u32, isize, usize?

## Features

### Parser

- [ ] Allow using global core variables in redcode, CORE_SIZE, MAX_INSTRUCTIONS etc.

### Core

- [ ] Helpers for running core, ie wrapping addition/subtraction, following indirect addressing and inc/decrementing ptrs.
- [ ] Actually process instructions!
- [ ] Report wins/losses
- [ ] Random address and separation
- [ ] Should running tournaments be in this lib, or the app?

### Logging

- [ ] Improve DebugLogger to collect ongoing stats or whatever.