use convert_case::{Case, Casing};

use super::tuple::Tuple;
use crate::astype::*;

pub trait IsNullable {
    fn is_nullable(&self) -> bool;
}

impl IsNullable for ASType {
    fn is_nullable(&self) -> bool {
        matches!(
            self,
            ASType::ConstPtr(_)
                | ASType::MutPtr(_)
                | ASType::ReadBuffer(_)
                | ASType::WriteBuffer(_)
                | ASType::Enum(_)
                | ASType::Struct(_)
                | ASType::Tuple(_)
                | ASType::Union(_)
        )
    }
}

pub trait Normalize {
    fn as_str(&self) -> &str;

    fn as_type(&self) -> String {
        self.as_str().to_case(Case::Pascal)
    }

    fn as_fn(&self) -> String {
        self.as_str().to_case(Case::Camel)
    }

    fn as_fn_suffix(&self) -> String {
        self.as_str().to_case(Case::UpperCamel)
    }

    fn as_var(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::Snake))
    }

    fn as_const(&self) -> String {
        escape_reserved_word(&self.as_str().to_case(Case::UpperSnake))
    }

    fn as_namespace(&self) -> String {
        let base_name = self.as_str();
        let ns_name = format!("{}_n",base_name);
        ns_name.as_str().to_case(Case::Pascal)
    }
}

impl<T: AsRef<str>> Normalize for T {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
}

pub trait ToLanguageRepresentation {
    fn as_astype(&self) -> &ASType;

    fn to_string(&self) -> String {
        self.as_lang()
    }

    fn as_lang(&self) -> String {
        self.as_lang_with_parent(None)
    }

    fn as_lang_with_parent(&self, parent_type: Option<&ASType>) -> String {
        match self.as_astype() {
            ASType::Alias(alias) => alias.name.as_type(),
            ASType::Bool => "bool".to_string(),
            ASType::Char32 => "string".to_string(),
            ASType::Char8 => "string".to_string(),
            ASType::F32 => "f32".to_string(),
            ASType::F64 => "f64".to_string(),
            ASType::Handle(_resource_name) => "Handle".to_string(),
            ASType::ConstPtr(pointee) => {
                match parent_type {
                    Some(ASType::Struct(_s)) => {format!("usize")}
                    _ => {format!("ptr<{}>", pointee.to_string())}
                }
            },
            ASType::MutPtr(pointee) => {
                match parent_type {
                    Some(ASType::Struct(_s)) => {format!("usize")}
                    _ => {format!("mutptr<{}>", pointee.to_string())}
                }
            }
            ASType::Option(_) => todo!(),
            ASType::Result(_) => todo!(),
            ASType::S8 => "i8".to_string(),
            ASType::S16 => "i16".to_string(),
            ASType::S32 => "i32".to_string(),
            ASType::S64 => "i64".to_string(),
            ASType::U8 => "u8".to_string(),
            ASType::U16 => "u16".to_string(),
            ASType::U32 => "u32".to_string(),
            ASType::U64 => "u64".to_string(),
            ASType::USize => "usize".to_string(),
            ASType::Void => "void".to_string(),
            ASType::Constants(_) => unimplemented!(),
            ASType::Enum(enum_) => {
                format!("{} /* Enum */", enum_.repr.as_ref().as_lang())
            }
            ASType::Struct(_) => unimplemented!(),
            ASType::Tuple(tuple_members) => Tuple::name_for(tuple_members).as_type(),
            ASType::Union(_) => unimplemented!(),
            ASType::Slice(element_type) => {
                match parent_type {
                    Some(t) => {
                        // this is only called in the case for typescript
                        format!("usize")
                    }
                    None => { format!("WasiMutSlice<{}>", element_type.as_lang()) }
                }
            }
            ASType::String(_) => "WasiString".to_string(),
            ASType::ReadBuffer(element_type) => {
                match parent_type {
                    Some(t) => {
                        // this is only called in the case for typescript
                        format!("usize")
                    }
                    None => { format!("WasiSlice<{}>", element_type.as_lang()) }
                }
            }
            ASType::WriteBuffer(element_type) => {
                match parent_type {
                    Some(t) => {
                        // this is only called in the case for typescript
                        format!("usize")
                    }
                    None => { format!("WasiMutSlice<{}>", element_type.as_lang()) }
                }
            }
        }
    }
}

impl ToLanguageRepresentation for ASType {
    fn as_astype(&self) -> &ASType {
        self
    }
}

pub fn escape_reserved_word(word: &str) -> String {
    //if word starts_with a digit
    if word.as_bytes()[0].is_ascii_digit() {
        format!("E_{}", word)
    } else {
        if RESERVED.iter().any(|k| *k == word) {
            // If the camel-cased string matched any strict or reserved keywords, then
            // append a trailing underscore to the identifier we generate.
            format!("{}_", word)
        } else {
            word.to_string() // Otherwise, use the string as is.
        }
    }
}

/// Reserved Keywords.
///
/// Source: [ECMAScript 2022 Language Specification](https://tc39.es/ecma262/#sec-keywords-and-reserved-words)
const RESERVED: &[&str] = &[
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
    "let",
    "static",
    "implements",
    "interface",
    "package",
    "private",
    "protected",
    "and",
    "public",
];
