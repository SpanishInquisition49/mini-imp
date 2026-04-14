#[macro_export]
macro_rules! register_pass {
    (
        $pass_type:ty {
            requires:[$($req:ty), * $(,)?],
            modifies:[$($mod:ty), * $(,)?],
        }
    ) => {
        impl $crate::optimisation::pass::PassMetadata for $pass_type {
            fn get_required_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationCheck> {
                vec![$($crate::data_flow::annotations::check::<$req>()), *]
            }
            fn get_modified_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationMark> {
                vec![$($crate::data_flow::annotations::mark::<$mod>()), *]
            }
        }
    };
    (
        $pass_type:ty {
            requires: [$($req:ty),* $(,)?],
        }
    ) => {
        impl $crate::optimisation::pass::PassMetadata for $pass_type {
            fn get_required_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationCheck> {
                vec![$($crate::data_flow::annotations::check::<$req>()),*]
            }
            fn get_modified_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationMark> {
                vec![]
            }
        }
    };
    (
        $pass_type:ty {
            modifies: [$($mod:ty),* $(,)?],
        }
    ) => {
        impl $crate::optimisation::pass::PassMetadata for $pass_type {
            fn get_required_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationCheck> {
                vec![]
            }
            fn get_modified_annotations(&self) -> Vec<$crate::data_flow::annotations::AnnotationMark> {
                vec![$($crate::data_flow::annotations::mark::<$mod>()),*]
            }
        }
    };
}
