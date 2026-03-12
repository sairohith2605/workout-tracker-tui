use std::fmt;
use uuid::Uuid;

macro_rules! uuid_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub Uuid);

        impl $name {
            #[allow(dead_code)]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn to_db_string(&self) -> String {
                self.0.hyphenated().to_string()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0.hyphenated())
            }
        }

        impl TryFrom<String> for $name {
            type Error = uuid::Error;
            fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
                Ok(Self(Uuid::parse_str(&s)?))
            }
        }
    };
}

uuid_newtype!(CategoryId);
uuid_newtype!(ExerciseId);
uuid_newtype!(SessionId);
uuid_newtype!(EntryId);
uuid_newtype!(SetId);

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exercise {
    pub id: ExerciseId,
    pub name: String,
    pub category_id: CategoryId,
    /// Denormalised category name for display (populated by JOIN in list query).
    pub category_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetInput {
    pub reps: u32,
    pub weight: f64,
}

/// An exercise being logged in the current (unsaved) workout session.
#[derive(Debug, Clone)]
pub struct SessionExercise {
    pub exercise: Exercise,
    pub sets: Vec<SetInput>,
}
