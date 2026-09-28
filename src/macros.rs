#[macro_export]
macro_rules! children {
    ($($child:expr),* $(,)?) => {
        vec![
            $(
                Box::new($child) as Box<dyn $crate::Widget>
            ),*
        ]
    };
}

#[macro_export]
macro_rules! shapes {
    ($($shape:expr),* $(,)?) => {
        vec![
            $(
                Box::new($shape) as Box<dyn $crate::Shape>
            ),*
        ]
    };
}

#[macro_export]
macro_rules! arguments {
    ($($argument:expr),* $(,)?) => {
        vec![
            $(
                $crate::Argument::from($argument)
            ),*
        ]
    };
}
