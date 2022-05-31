//! Dear fellow macro enjoyer,
//!
//! You'll find, once you look at this design
//! A huge piece of very complex code,
//! It takes tokens - all written inline,
//! Turns it into appropriate node.
//!
//! The next few sentences shall explain,
//! What's complex, cursed, or damn simple,
//! So you don't feel lost and get less pain,
//! Once you start diving in this `impl`.
//!
//! You'll see some simple conventions there,
//! So that everything stays consistent,
//! And it's little less than a nightmare,
//! To dive in this desolate content.
//!
//! # Ok Sasha, but I'm looking for the docs
//!
//! The complexity of the inline macros has been split across three different
//! submodules: `expression`, `function` and `program`. The `node` submodule
//! contains free functions aiming to make test AST creation as simple as
//! possible. Let's forget this last module in the rest of the explanation.
//!
//! Each module (we will call it `$mod`) defines at least two macros:
//! `inline_$mod` and `parse_$mod`. `inline_$mod` is the one that should be
//! called directly by the outside user. It calls `parse_$mod` under the hood,
//! and perform some additional conversion. These two macros should take all the
//! tokens inline, with no specific syntax.
//!
//! If the grammar recognized by `parse_$mod` is simple enough, the whole
//! parsing step may happen at `parse_$mod`. Otherwise, `parse_$mod` acts like
//! a simple wrapper over `parse_$mod_inner`, calling it with the appropriate
//! token trees.
//!
//! Declarative macros take tokens trees as argument. We're processing them one
//! token at a time, taking the leftmost one, doing the appropriate
//! transformations and putting it at the end of a buffer, then recursing, until
//! we either meet an interesting condition or have no more tokens to consume.
//! The tokens that are to be consumed are always passed first, enclosed
//! brackets. Then are passed the tokens we have just processed. We sometimes
//! have to specify which state we're in. This state name always starts with an
//! `@` and is matched *even before* the unprocessed tokens. When there's a
//! clear separation, that state can even be put in *another macro*.

pub(crate) mod expression;
pub(crate) mod function;
pub(crate) mod nodes;
pub(crate) mod program;

#[macro_export]
macro_rules! node {
    ($node_name:ident( $( $args:tt )* )) => {
        $crate::macros::nodes::$node_name( $( $args )* )
    };
}
