use std::io::Read;
use fbx_binary_reader::{EventReader, Property, PropertiesIter};
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use super::{PropertyFlags, PropertyNodeValue};


#[derive(Debug, Clone)]
pub struct PropertyNode {
    pub type_name: String,
    pub label: String,
    pub flags: PropertyFlags,
    pub value: PropertyNodeValue,
}

#[derive(Debug)]
pub struct PropertyNodeLoader<'a> {
    type_name: &'a str,
    label: &'a str,
    flags: PropertyFlags,
    value: Option<PropertyNodeValue>,
}

impl<'a> PropertyNodeLoader<'a> {
    pub fn new(mut iter: PropertiesIter<'a>) -> Option<Self> {
        let type_name = if let Some(val) = iter.next().and_then(|p| p.get_string()) {
            val
        } else {
            error!("Cannot get property node type name");
            return None;
        };
        let label = if let Some(val) = iter.next().and_then(|p| p.get_string()) {
            val
        } else {
            error!("Cannot get property node label");
            return None;
        };
        let flags = if let Some(val) = iter.next().and_then(|p| p.get_string()) {
            PropertyFlags::from_string(val)
        } else {
            error!("Cannot get property node flags");
            return None;
        };
        let value = match label {
            "Blob" => {
                if let Some(length) = iter.next().and_then(|p| p.get_i32()) {
                    Some(PropertyNodeValue::Blob(Vec::with_capacity(length as usize)))
                } else {
                    error!("Cannot get length of a binary property node");
                    return None;
                }
            },
            _ => {
                iter.next().map_or(Some(PropertyNodeValue::Empty), |first_val| {
                    Some(match first_val {
                        Property::I16(val) => {
                            let mut vec = vec![val as i64];
                            vec.extend(iter.scan((), |_, p| p.as_i64()));
                            if vec.len() == 1 {
                                PropertyNodeValue::I64(val as i64)
                            } else {
                                PropertyNodeValue::VecI64(vec)
                            }
                        },
                        Property::I32(val) => {
                            let mut vec = vec![val as i64];
                            vec.extend(iter.scan((), |_, p| p.as_i64()));
                            if vec.len() == 1 {
                                PropertyNodeValue::I64(val as i64)
                            } else {
                                PropertyNodeValue::VecI64(vec)
                            }
                        },
                        Property::I64(val) => {
                            let mut vec = vec![val];
                            vec.extend(iter.scan((), |_, p| p.as_i64()));
                            if vec.len() == 1 {
                                PropertyNodeValue::I64(val)
                            } else {
                                PropertyNodeValue::VecI64(vec)
                            }
                        },
                        Property::F32(val) => {
                            let mut vec = vec![val];
                            vec.extend(iter.scan((), |_, p| p.as_f32()));
                            if vec.len() == 1 {
                                PropertyNodeValue::F32(val)
                            } else {
                                PropertyNodeValue::VecF32(vec)
                            }
                        },
                        Property::F64(val) => {
                            let mut vec = vec![val];
                            vec.extend(iter.scan((), |_, p| p.as_f64()));
                            if vec.len() == 1 {
                                PropertyNodeValue::F64(val)
                            } else {
                                PropertyNodeValue::VecF64(vec)
                            }
                        },
                        // Vec<String> does not seem to exist.
                        Property::String(val) => PropertyNodeValue::String(val.map(|v| v.to_owned()).map_err(Into::into)),
                        val => {
                            // Unexpected type. Discard the property.
                            // (bool, Vec<bool> and Vec<Vec<_>> don't seems to exist.)
                            // (boolean value is represented with integer in property node.)
                            error!("Unexpected (unsupported) property node value: {:?}", val);
                            return None;
                        },
                    })
                })
            },
        };
        Some(PropertyNodeLoader {
            type_name: type_name,
            label: label,
            flags: flags,
            value: value,
        })
    }
}

impl<'a> NodeLoaderCommon for PropertyNodeLoader<'a> {
    type Target = Option<PropertyNode>;

    fn on_finish(self) -> Result<Self::Target> {
        let PropertyNodeLoader { type_name, label, flags, value } = self;
        Ok(value.map(|value| PropertyNode {
            type_name: type_name.to_owned(),
            label: label.to_owned(),
            flags: flags,
            value: value,
        }))
    }
}

impl<'a, R: Read> NodeLoader<R> for PropertyNodeLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        if self.type_name == "Blob" {
            match name.as_ref() {
                "BinaryData" => {
                    if let Some(binary) = properties.iter().next().and_then(|p| p.get_binary()) {
                        match self.value {
                            Some(PropertyNodeValue::Blob(ref mut vec)) => {
                                vec.extend(binary);
                            },
                            Some(_) => unreachable!(),
                            None => {},
                        }
                    } else {
                        error!("Invalid node property: Cannot get binary data from `P/BinaryData`");
                    }
                },
                _ => {
                    warn!("Unknown node: `P/{}`", name);
                },
            }
        } else {
            warn!("Unnecessary child node: `P/{}`", name);
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
