//! Adversarial authorization negative-test matrix.
//!
//! This crate contains no production code. It exists so the workspace can host
//! cross-contract authorization scenarios that no single contract crate can
//! express on its own: the admin records live in three separate contracts, and
//! `proof-registry` gates its two revocation paths on *different* identities
//! (the proof's issuer vs. the registry admin).
//!
//! The matrix is documented in `docs/authorization-matrix.md`; every assertion
//! maps to a row of that document, and a count guard fails when the public
//! surface drifts from it.
//!
//! Unlike the rest of the repository, nothing in this crate calls
//! `mock_all_auths` for the negative cases. The harness authorizes each call
//! with matching-mode `mock_auths` entries, so a call signed by the wrong
//! identity is actually *rejected* by the host instead of silently admitted.
//! That is what makes "unauthorized calls change no storage, TTL, event, or
//! cross-contract state" an executable claim rather than a documented one.
//!
//! Privacy note: these tests deliberately use synthetic hashes and addresses.
//! No fixture encodes a real wallet, a real proof identifier, or any off-chain
//! payload.

#![no_std]

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod harness;

#[cfg(test)]
mod matrix;

#[cfg(test)]
mod rotation;

#[cfg(test)]
mod delegation;

#[cfg(test)]
mod enumeration;

#[cfg(test)]
mod probe;
