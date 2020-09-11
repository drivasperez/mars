#![forbid(unsafe_code)]
//! # A rust library emulating a CoreWar engine
//!
//! Something something '95 standard, very cool, really great, try it some time.
//!
//! ## Using this library
//!
//! You probably want to parse some [Warriors](warrior/struct.Warrior.html) and put them in a
//! [Core](core/struct.Core.html).
//!
//! ## Writing a warrior
//!
//! Warriors are written in a pseudo-assembly language called Redcode. Below is a simple warrior:
//!
//! ```redcode
//! ;redcode-94
//! ;name Imp
//! ;author A.K. Dewdney
//!
//!         org     imp
//!
//! imp:    mov.i   imp, imp+1
//!         end
//! ```
//!
pub mod core;
pub mod error;
pub mod executor;
pub(crate) mod parser;
pub mod warrior;
