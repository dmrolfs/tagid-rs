use crate::Label;
use once_cell::sync::OnceCell;
use pretty_type_name::pretty_type_name;
use smol_str::SmolStr;
use std::convert::Infallible;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

pub trait Labeling {
    fn label(&self) -> &str;
}

impl dyn Labeling {
    /// Summon an instance of the Labeler for T.
    pub fn summon<T: Label>() -> <T as Label>::Labeler {
        T::labeler()
    }
}

#[derive(Clone)]
pub struct MakeLabeling<T: ?Sized> {
    label: OnceCell<SmolStr>,
    marker: PhantomData<T>,
}

impl<T: ?Sized> MakeLabeling<T> {
    pub const fn new() -> Self {
        Self {
            label: OnceCell::new(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Default for MakeLabeling<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Labeling for MakeLabeling<T> {
    fn label(&self) -> &str {
        self.label
            .get_or_init(|| SmolStr::new(pretty_type_name::<T>()))
            .as_str()
    }
}

impl<T: ?Sized> fmt::Debug for MakeLabeling<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MakeLabeling({})", self.label())
    }
}

impl<T: ?Sized> fmt::Display for MakeLabeling<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Clone)]
pub struct CustomLabeling {
    label: SmolStr,
}

impl CustomLabeling {
    pub fn new(label: impl AsRef<str>) -> Self {
        Self {
            label: SmolStr::new(label),
        }
    }
}

impl Labeling for CustomLabeling {
    fn label(&self) -> &str {
        self.label.as_str()
    }
}

impl fmt::Debug for CustomLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomLabeling({})", self.label())
    }
}

impl fmt::Display for CustomLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl From<&str> for CustomLabeling {
    fn from(label: &str) -> Self {
        Self {
            label: SmolStr::new(label),
        }
    }
}

impl From<String> for CustomLabeling {
    fn from(label: String) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl FromStr for CustomLabeling {
    type Err = Infallible;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        Ok(label.into())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NoLabeling;

impl Labeling for NoLabeling {
    fn label(&self) -> &str {
        ""
    }
}

impl fmt::Display for NoLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
