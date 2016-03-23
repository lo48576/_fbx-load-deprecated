use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum PropertyNodeValue {
    Empty,
    Blob(Vec<u8>),
    String(Result<String, Vec<u8>>),
    F32(f32),
    F64(f64),
    VecF32(Vec<f32>),
    VecF64(Vec<f64>),
    I64(i64),
    VecI64(Vec<i64>),
}

impl PropertyNodeValue {
    pub fn get_vec_u8(&self) -> Option<&Vec<u8>> {
        match *self {
            PropertyNodeValue::Blob(ref val) => Some(&val),
            _ => None,
        }
    }

    pub fn into_vec_u8(self) -> Result<Vec<u8>, Self> {
        match self {
            PropertyNodeValue::Blob(val) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_string(&self) -> Option<&String> {
        match *self {
            PropertyNodeValue::String(Ok(ref val)) => Some(&val),
            _ => None,
        }
    }

    pub fn into_string(self) -> Result<String, Self> {
        match self {
            PropertyNodeValue::String(Ok(val)) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_string_or_raw(&self) -> Option<&Result<String, Vec<u8>>> {
        match *self {
            PropertyNodeValue::String(ref val) => Some(&val),
            _ => None,
        }
    }

    pub fn into_string_or_raw(self) -> Result<Result<String, Vec<u8>>, Self> {
        match self {
            PropertyNodeValue::String(val) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_f32(&self) -> Option<f32> {
        match *self {
            PropertyNodeValue::F32(val) => Some(val),
            PropertyNodeValue::F64(val) => Some(val as f32),
            _ => None,
        }
    }

    pub fn into_f32(self) -> Result<f32, Self> {
        match self {
            PropertyNodeValue::F32(val) => Ok(val),
            PropertyNodeValue::F64(val) => Ok(val as f32),
            val => Err(val),
        }
    }

    pub fn get_f64(&self) -> Option<f64> {
        match *self {
            PropertyNodeValue::F32(val) => Some(val as f64),
            PropertyNodeValue::F64(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_f64(self) -> Result<f64, Self> {
        match self {
            PropertyNodeValue::F32(val) => Ok(val as f64),
            PropertyNodeValue::F64(val) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_vec_f32(&self) -> Option<Cow<[f32]>> {
        match *self {
            PropertyNodeValue::VecF32(ref val) => Some(Cow::Borrowed(&val)),
            PropertyNodeValue::VecF64(ref val) => Some(Cow::Owned(val.iter().map(|&v| v as f32).collect())),
            _ => None,
        }
    }

    pub fn into_vec_f32(self) -> Result<Vec<f32>, Self> {
        match self {
            PropertyNodeValue::VecF32(val) => Ok(val),
            PropertyNodeValue::VecF64(val) => Ok(val.into_iter().map(|v| v as f32).collect()),
            val => Err(val),
        }
    }

    pub fn get_vec_f64(&self) -> Option<Cow<[f64]>> {
        match *self {
            PropertyNodeValue::VecF32(ref val) => Some(Cow::Owned(val.iter().map(|&v| v as f64).collect())),
            PropertyNodeValue::VecF64(ref val) => Some(Cow::Borrowed(&val)),
            _ => None,
        }
    }

    pub fn into_vec_f64(self) -> Result<Vec<f64>, Self> {
        match self {
            PropertyNodeValue::VecF32(val) => Ok(val.into_iter().map(|v| v as f64).collect()),
            PropertyNodeValue::VecF64(val) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_i64(&self) -> Option<i64> {
        match *self {
            PropertyNodeValue::I64(val) => Some(val),
            _ => None,
        }
    }

    pub fn into_i64(self) -> Result<i64, Self> {
        match self {
            PropertyNodeValue::I64(val) => Ok(val),
            val => Err(val),
        }
    }

    pub fn get_vec_i64(&self) -> Option<&Vec<i64>> {
        match *self {
            PropertyNodeValue::VecI64(ref val) => Some(&val),
            _ => None,
        }
    }

    pub fn into_vec_i64(self) -> Result<Vec<i64>, Self> {
        match self {
            PropertyNodeValue::VecI64(val) => Ok(val),
            val => Err(val),
        }
    }
}
