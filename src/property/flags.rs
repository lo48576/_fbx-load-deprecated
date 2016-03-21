//! Contains flags type for `Propertiesxx` (`xx` is 60, 70, ...) nodes.

/// Flags of property node.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PropertyFlags(u32);

impl PropertyFlags {
    pub fn none() -> Self {
        PropertyFlags(0_u32)
    }

    pub fn set_animatable(self, value: bool) -> Self {
        const ANIMATABLE: u32 = 1_u32 << 1;
        PropertyFlags(if value {
            self.0 | ANIMATABLE
        } else {
            self.0 & !ANIMATABLE
        })
    }

    pub fn set_animated(self, value: bool) -> Self {
        const ANIMATABLE: u32 = 1_u32 << 1;
        const ANIMATED: u32 = 1_u32 << 2;
        PropertyFlags(if value {
            self.0 | ANIMATABLE | ANIMATED
        } else {
            self.0 & !ANIMATED
        })
    }

    pub fn set_user_defined(self, value: bool) -> Self {
        const USER_DEFINED: u32 = 1_u32 << 4;
        PropertyFlags(if value {
            self.0 | USER_DEFINED
        } else {
            self.0 & !USER_DEFINED
        })
    }

    pub fn set_hidden(self, value: bool) -> Self {
        const HIDDEN: u32 = 1_u32 << 4;
        PropertyFlags(if value {
            self.0 | HIDDEN
        } else {
            self.0 & !HIDDEN
        })
    }

    pub fn set_locked_member(self, members: u8) -> Self {
        const LOCKED_MEMBER_SHIFT: u8 = 7;
        assert!(members <= 15, "Value for LockedMember flag should be 0-15");
        // FIXME: How members over 9 is represented?
        if members >= 10 {
            panic!("member>=10 unsupported (specification for LockedMember is unknown)");
        }
        PropertyFlags((self.0 & 15 << LOCKED_MEMBER_SHIFT) | ((members as u32) << LOCKED_MEMBER_SHIFT))
    }

    pub fn from_string(flags_str: &str) -> Self {
        let mut ret = Self::none();
        let mut chars = flags_str.chars().peekable();
        let mut c = chars.next();
        loop {
            match c {
                Some('U') => {
                    ret = ret.set_user_defined(true);
                    c = chars.next();
                },
                Some('A') => {
                    ret = ret.set_animatable(true);
                    c = chars.next();
                    if c == Some('+') {
                        ret = ret.set_animated(true);
                        c = chars.next();
                    }
                },
                Some('H') => {
                    ret = ret.set_hidden(true);
                    c = chars.next();
                },
                Some('L') => {
                    c = chars.next();
                    // FIXME: How members over 9 is represented?
                    match c {
                        Some(code @ '0'...'9') => {
                            ret = ret.set_locked_member((code as u32 - '0' as u32) as u8);
                            c = chars.next();
                        },
                        _ => {},
                    }
                },
                Some(val) => {
                    warn!("Unknown character as proprety flags: {:?}", val);
                    c = chars.next();
                }
                None => {
                    break;
                },
            }
        }
        ret
    }
}

#[cfg(tests)]
mod property_flag_test {
    #[test]
    fn property_flags_animated_and_animatable() {
        // `set_animated(true)` should turn `Animatable` on.
        assert_eq!(
            PropertyFlags::none().set_animated(true),
            PropertyFlags::none().set_animated(true).set_animatable(true));

        // set_animated(false)` shouldn't turn `Animatable` off.
        assert_eq!(
            PropertyFlags::none().set_animatable(true),
            PropertyFlags::none().set_animatable(true).set_animated(false));
    }

    #[test]
    fn property_flags_from_string() {
        assert_eq!(
            PropertyFlags::from_string("UA+HL7"),
            PropertyFlags::none().set_user_defined(true).set_animated(true).set_hidden(true).set_locked_member(7));

        assert_eq!(
            PropertyFlags::from_string("AHL3"),
            PropertyFlags::none().set_animatable(true).set_hidden(true).set_locked_member(3));
    }
}
