//! Contians properties common to the FBX objects.

use fbx_binary_reader::PropertiesIter;
use ::separate_name_class;

#[derive(Debug, Clone)]
pub struct ObjectProperties<'a> {
    pub id: i64,
    pub name: &'a str,
    pub class: &'a str,
    pub subclass: &'a str,
}

impl<'a> ObjectProperties<'a> {
    pub fn from_node_properties(mut iter: PropertiesIter<'a>) -> Option<Self> {
        let id = iter.next().and_then(|p| p.get_i64());
        let name_class = iter.next().and_then(|p| p.get_string()).and_then(separate_name_class);
        let subclass = iter.next().and_then(|p| p.get_string());
        if let (Some(id), Some((name, class)), Some(subclass)) = (id, name_class, subclass) {
            Some(ObjectProperties {
                id: id,
                name: name,
                class: class,
                subclass: subclass,
            })
        } else {
            error!("Cannot get object propeties");
            None
        }
    }
}
