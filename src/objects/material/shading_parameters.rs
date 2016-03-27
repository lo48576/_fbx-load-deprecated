//! Shading parameters of materials.

use definitions::PropertyTemplates;
use property::{GenericProperties, OptionalProperties};

#[derive(Debug, Clone)]
pub enum ShadingParameters {
    Lambert(LambertParameters),
    Phong(PhongParameters),
    Unknown(Option<GenericProperties>),
}

impl ShadingParameters {
    pub fn from_node_properties<S: AsRef<str>>(shading_model: S, properties: &mut Option<GenericProperties>, property_templates: &PropertyTemplates) -> Self {
        match shading_model.as_ref() {
            "lambert" => ShadingParameters::Lambert(LambertParameters::from_node_properties(properties, property_templates)),
            "phong" => ShadingParameters::Phong(PhongParameters::from_node_properties(properties, property_templates)),
            val => {
                warn!("Shading model `{}` is unknown and unsupported", val);
                ShadingParameters::Unknown(properties.take())
            },
        }
    }
}

/// Parameters for Lambert shading.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LambertParameters {
    /// `EmissiveColor` (not `Emissive`)
    pub emissive: [f32; 3],
    /// `EmissiveFactor`
    pub emissive_factor: f32,
    /// `AmbientColor` (not `Ambient`)
    pub ambient: [f32; 3],
    /// `AmbientFactor`
    pub ambient_factor: f32,
    /// `DiffuseColor` (not `Diffuse`)
    pub diffuse: [f32; 3],
    /// `DiffuseFactor`
    pub diffuse_factor: f32,
    /// `NormalMap`
    pub normal_map: [f32; 3],
    /// `Bump`
    pub bump: [f32; 3],
    /// `TransparentColor`
    pub transparent_color: [f32; 3],
    /// `TransparencyFactor`
    pub transparency_factor: f32,
    /// `DisplacementColor`
    pub displacement_color: [f32; 3],
    /// `DisplacementFactor`
    pub displacement_factor: f32,
    /// `VectorDisplacementColor`
    pub vector_displacement_color: [f32; 3],
    /// `VectorDisplacementFactor`
    pub vector_displacement_factor: f32,
}

impl LambertParameters {
    pub fn from_node_properties(properties: &mut Option<GenericProperties>, property_templates: &PropertyTemplates) -> Self {
        let mut params: Self = Default::default();
        let defaults = property_templates.templates.get(&("Material".to_owned(), "FbxSurfaceLambert".to_owned())).map(|t| &t.properties);
        let load_color = |props: &mut Option<GenericProperties>, key: &str, target: &mut [f32; 3]| {
            props.get_or_default(defaults, key).and_then(|p| p.value.get_vec_f32().into_iter().find(|v| v.len() >= 3).map(|v| [v[0], v[1], v[2]])).map(|v| *target = v);
        };
        let load_f32 = |props: &mut Option<GenericProperties>, key: &str, target: &mut f32| {
            props.get_or_default(defaults, key).and_then(|p| p.value.get_f32()).map(|v| *target = v);
        };

        load_color(properties, "EmissiveColor", &mut params.emissive);
        load_f32(properties, "EmissiveFactor", &mut params.emissive_factor);
        load_color(properties, "AmbientColor", &mut params.ambient);
        load_f32(properties, "AmbientFactor", &mut params.ambient_factor);
        load_color(properties, "DiffuseColor", &mut params.diffuse);
        load_f32(properties, "DiffuseFactor", &mut params.diffuse_factor);
        load_color(properties, "NormalMap", &mut params.normal_map);
        load_color(properties, "Bump", &mut params.bump);
        load_color(properties, "TransparentColor", &mut params.transparent_color);
        load_f32(properties, "TransparencyFactor", &mut params.transparency_factor);
        load_color(properties, "DisplacementColor", &mut params.displacement_color);
        load_f32(properties, "DisplacementFactor", &mut params.displacement_factor);
        load_color(properties, "VectorDisplacementColor", &mut params.vector_displacement_color);
        load_f32(properties, "VectorDisplacementFactor", &mut params.vector_displacement_factor);

        params
    }
}

/// Parameters for Lambert shading.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PhongParameters {
    /// Parameters common with lambert shading.
    pub lambert: LambertParameters,
    /// `SpecularColor` (not `Specular`)
    pub specular: [f32; 3],
    /// `SpecularFactor`
    pub specular_factor: f32,
    /// `Shininess`
    pub shininess: f32,
    /// `ReflectionColor` (not `Reflection`)
    pub reflection: [f32; 3],
    /// `ReflectionFactor`
    pub reflection_factor: f32,
}

impl PhongParameters {
    pub fn from_node_properties(properties: &mut Option<GenericProperties>, property_templates: &PropertyTemplates) -> Self {
        let mut params: Self = Default::default();
        let defaults = property_templates.templates.get(&("Material".to_owned(), "FbxSurfacePhong".to_owned())).map(|t| &t.properties);
        let load_color = |props: &mut Option<GenericProperties>, key: &str, target: &mut [f32; 3]| {
            props.get_or_default(defaults, key).and_then(|p| p.value.get_vec_f32().into_iter().find(|v| v.len() >= 3).map(|v| [v[0], v[1], v[2]])).map(|v| *target = v);
        };
        let load_f32 = |props: &mut Option<GenericProperties>, key: &str, target: &mut f32| {
            props.get_or_default(defaults, key).and_then(|p| p.value.get_f32()).map(|v| *target = v);
        };

        params.lambert = LambertParameters::from_node_properties(properties, property_templates);
        load_color(properties, "SpecularColor", &mut params.specular);
        load_f32(properties, "SpecularFactor", &mut params.specular_factor);
        load_f32(properties, "Shininess", &mut params.shininess);
        load_color(properties, "ReflectionColor", &mut params.reflection);
        load_f32(properties, "ReflectionFactor", &mut params.reflection_factor);

        params
    }
}
