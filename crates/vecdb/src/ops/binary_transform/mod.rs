mod reverse_operands;

pub use reverse_operands::ReverseOperands;

/// A binary operation passed by type (usually a zero-sized struct): calls are statically
/// dispatched.
pub trait BinaryTransform<In1, In2, Out = In1> {
    fn apply(lhs: In1, rhs: In2) -> Out;
}
