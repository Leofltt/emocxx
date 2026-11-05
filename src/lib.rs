pub mod compiler;
pub mod depfile;
pub mod lexer;
pub mod mangler;
pub mod shadow;

pub use compiler::CompilerInvocation;
pub use lexer::{transpile, transpile_source};
pub use mangler::{is_emoji_codepoint, is_emoji_grapheme, mangle_grapheme};
pub use shadow::ShadowCache;

