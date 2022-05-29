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
