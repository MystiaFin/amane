#[macro_export]
macro_rules! children {
    ($($child:expr),* $(,)? => {
        vec![
            $(
                Box::new($child) as Box<dyn $crate::Widget>
            )
        ]
    };
    )
}
