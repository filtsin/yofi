use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Mul;
use std::str::FromStr;

use serde::de::{Deserializer, Visitor};
use serde::Deserialize;

#[derive(Clone, Default)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Default)]
pub struct Margin {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Default)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl Padding {
    pub const fn all(val: f32) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }

    pub const fn from_pair(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

impl Mul<f32> for &Padding {
    type Output = Padding;

    fn mul(self, rhs: f32) -> Padding {
        Padding {
            top: self.top * rhs,
            bottom: self.bottom * rhs,
            left: self.left * rhs,
            right: self.right * rhs,
        }
    }
}

impl Margin {
    pub const fn all(val: f32) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }

    pub const fn from_pair(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
}

impl Mul<f32> for &Radius {
    type Output = Radius;

    fn mul(self, rhs: f32) -> Radius {
        Radius {
            top_left: self.top_left * rhs,
            top_right: self.top_right * rhs,
            bottom_left: self.bottom_left * rhs,
            bottom_right: self.bottom_right * rhs,
        }
    }
}

impl Radius {
    pub const fn all(val: f32) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val,
        }
    }

    pub const fn from_pair(first: f32, second: f32) -> Self {
        Self {
            top_left: first,
            top_right: second,
            bottom_left: second,
            bottom_right: first,
        }
    }

    pub(crate) fn min(&self, other: Radius) -> Radius {
        Self {
            top_left: self.top_left.min(other.top_left),
            top_right: self.top_right.min(other.top_right),
            bottom_left: self.bottom_left.min(other.bottom_left),
            bottom_right: self.bottom_right.min(other.bottom_right),
        }
    }
}

impl Mul<f32> for &Margin {
    type Output = Margin;

    fn mul(self, rhs: f32) -> Margin {
        Margin {
            top: self.top * rhs,
            bottom: self.bottom * rhs,
            left: self.left * rhs,
            right: self.right * rhs,
        }
    }
}

impl<'de> Deserialize<'de> for Padding {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(StringVisitor(PhantomData))
    }
}

impl<'de> Deserialize<'de> for Margin {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(StringVisitor(PhantomData))
    }
}

impl<'de> Deserialize<'de> for Radius {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(StringVisitor(PhantomData))
    }
}

struct StringVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for StringVisitor<T>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("unexpected value")
    }

    fn visit_str<E>(self, value: &str) -> Result<T, E>
    where
        E: serde::de::Error,
    {
        FromStr::from_str(value).map_err(serde::de::Error::custom)
    }
}

struct FiniteFloatVec {
    pub values: Vec<f32>,
}

impl FromStr for FiniteFloatVec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = s
            .split(' ')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<f32>()
                    .map_err(|_| format!("invalid float value: {:?}", s))
                    .and_then(|f| {
                        f.is_finite()
                            .then_some(f)
                            .ok_or(format!("non-finite float value: {:?}", f))
                    })
            })
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(Self { values })
    }
}

impl FromStr for Padding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = FiniteFloatVec::from_str(s)?.values;

        match values.len() {
            1 => Ok(Self::all(values[0])),
            2 => Ok(Self::from_pair(values[0], values[1])),
            4 => Ok(Self {
                top: values[0],
                bottom: values[2],
                left: values[3],
                right: values[1],
            }),
            _ => Err("padding should consists of either 1, 2 or 4 floats".into()),
        }
    }
}

impl FromStr for Margin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = FiniteFloatVec::from_str(s)?.values;

        match values.len() {
            1 => Ok(Self::all(values[0])),
            2 => Ok(Self::from_pair(values[0], values[1])),
            4 => Ok(Self {
                top: values[0],
                bottom: values[2],
                left: values[3],
                right: values[1],
            }),
            _ => Err("margin should consists of either 1, 2 or 4 floats".into()),
        }
    }
}

impl FromStr for Radius {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let values = FiniteFloatVec::from_str(s)?
            .values
            .iter()
            .map(|&f| {
                (!f.is_sign_negative())
                    .then_some(f)
                    .ok_or(format!("radius can not be negative: {:?}", f))
            })
            .collect::<Result<Vec<f32>, _>>()?;

        match values.len() {
            1 => Ok(Self::all(values[0])),
            2 => Ok(Self::from_pair(values[0], values[1])),
            4 => Ok(Self {
                top_left: values[0],
                top_right: values[1],
                bottom_left: values[3],
                bottom_right: values[2],
            }),
            _ => Err("radius should consists of either 1, 2 or 4 floats".into()),
        }
    }
}
