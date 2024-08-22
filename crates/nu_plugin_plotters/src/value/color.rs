use nu_protocol::{FromValue, IntoValue, ShellError, Span, SyntaxShape, Type, Value};
use plotters::style::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, IntoValue, Serialize, Deserialize)]
pub struct Color {
    pub r: ColorChannel,
    pub g: ColorChannel,
    pub b: ColorChannel,
    pub a: AlphaChannel,
}

impl FromValue for Color {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        match v {
            val @ Value::Record { .. } => {
                #[derive(FromValue)]
                struct ColorDTO {
                    r: ColorChannel,
                    g: ColorChannel,
                    b: ColorChannel,
                    a: Option<AlphaChannel>,
                }

                let color = ColorDTO::from_value(val)?;
                Ok(Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a.unwrap_or_default(),
                })
            }

            list @ Value::List { .. } => {
                let rgba = <(ColorChannel, ColorChannel, ColorChannel, AlphaChannel)>::from_value(
                    list.clone(),
                );
                let rgb = <(ColorChannel, ColorChannel, ColorChannel)>::from_value(list);
                match (rgba, rgb) {
                    (Ok((r, g, b, a)), _) => Ok(Color { r, g, b, a }),
                    (Err(ShellError::CantFindColumn { .. }), Ok((r, g, b))) => Ok(Color {
                        r,
                        g,
                        b,
                        a: Default::default(),
                    }),
                    (Err(e), Ok(_)) => Err(e),
                    (Err(_), Err(e)) => Err(e),
                }
            }

            ref v @ Value::String { ref val, .. } => match val.as_str() {
                "black" => Ok(BLACK.into()),
                "blue" => Ok(BLUE.into()),
                "cyan" => Ok(CYAN.into()),
                "green" => Ok(GREEN.into()),
                "magenta" => Ok(MAGENTA.into()),
                "red" => Ok(RED.into()),
                "white" => Ok(WHITE.into()),
                "yellow" => Ok(YELLOW.into()),
                _ => Err(ShellError::CantConvert {
                    to_type: Self::expected_type().to_string(),
                    from_type: v.get_type().to_string(),
                    span: v.span(),
                    help: None,
                }),
            },

            v => Err(ShellError::CantConvert {
                to_type: Self::expected_type().to_string(),
                from_type: v.get_type().to_string(),
                span: v.span(),
                help: None,
            }),
        }
    }

    fn expected_type() -> Type {
        Type::Custom("plotters::color".to_string().into_boxed_str())
    }
}

impl From<RGBColor> for Color {
    fn from(value: RGBColor) -> Self {
        Self {
            r: ColorChannel(value.0),
            g: ColorChannel(value.1),
            b: ColorChannel(value.2),
            a: Default::default(),
        }
    }
}

impl From<Color> for plotters::style::RGBAColor {
    fn from(value: Color) -> Self {
        Self(value.r.0, value.g.0, value.b.0, value.a.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorChannel(u8);

impl IntoValue for ColorChannel {
    fn into_value(self, span: Span) -> Value {
        Value::int(self.0 as i64, span)
    }
}

impl FromValue for ColorChannel {
    fn from_value(value: Value) -> Result<Self, ShellError> {
        let span = value.span();
        let value = i64::from_value(value)?;
        const U8MIN: i64 = u8::MIN as i64;
        const U8MAX: i64 = u8::MAX as i64;
        #[allow(overlapping_range_endpoints)]
        match value {
            U8MIN..=U8MAX => Ok(ColorChannel(value as u8)),
            i64::MIN..U8MIN => Err(ShellError::GenericError {
                error: "Integer too small".to_string(),
                msg: format!("{value} is smaller than {U8MIN}"),
                span: Some(span),
                help: None,
                inner: vec![],
            }),
            U8MAX..=i64::MAX => Err(ShellError::GenericError {
                error: "Integer too large".to_string(),
                msg: format!("{value} is larger than {U8MAX}"),
                span: Some(span),
                help: None,
                inner: vec![],
            }),
        }
    }

    fn expected_type() -> Type {
        Type::Int
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlphaChannel(f64);

impl IntoValue for AlphaChannel {
    fn into_value(self, span: Span) -> Value {
        Value::float(self.0, span)
    }
}

impl FromValue for AlphaChannel {
    fn from_value(v: Value) -> Result<Self, ShellError> {
        let span = v.span();
        let v = f64::from_value(v)?;
        match v {
            0.0..=1.0 => Ok(Self(v)),
            _ => Err(ShellError::GenericError {
                error: "Invalid alpha channel value".to_string(),
                msg: format!("{v} is not in range between 0.0 and 1.0"),
                span: Some(span),
                help: None,
                inner: vec![],
            }),
        }
    }

    fn expected_type() -> nu_protocol::Type {
        f64::expected_type()
    }
}

impl Default for AlphaChannel {
    fn default() -> Self {
        Self(1.0)
    }
}
