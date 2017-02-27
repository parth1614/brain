use parser::{Expression};
use memory::MemoryBlock;

use super::{Operation};
use super::scope::ScopeStack;

/// Generates operations for evaluating the given expression
/// and storing its result in the given destination memory block
pub fn into_operations(
    expr: Expression,
    destination: MemoryBlock,
    scope: &mut ScopeStack,
) -> Vec<Operation> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;
}
